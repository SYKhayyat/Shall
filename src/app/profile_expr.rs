// src/app/profile_expr.rs
//
// A tiny, dependency-free set-expression language for composing profiles.
//
// Motivation: profiles already support line directives (`include`/`exclude`/`-pkg`), which
// give union and subtraction. This module adds *intersection* and *arbitrary grouping* so a
// user can express things like "the union of Work and Gaming, intersected with Security":
//
//     intersect(union(work, gaming), security)
//     (work | gaming) & security          # infix form, equivalent
//
// Design decisions:
// - Operators are `|` (union), `&` (intersect), `\` (difference), plus the function forms
//   `union(...)`, `intersect(...)`, `diff(...)`. Parentheses group, and nest infinitely.
// - We deliberately do NOT use `+`/`-` as infix operators: real package atoms contain them
//   (`g++`, `libstdc++`, `apt:foo-bar`), so an infix `+` would be ambiguous. `|`/`&`/`\`
//   never appear in package names, so tokenizing is unambiguous.
// - Precedence: `&` binds tighter than `|`/`\` (both lowest, left-associative). So
//   `a | b & c` == `a | (b & c)`; write `(a | b) & c` for the other grouping. This mirrors
//   boolean algebra and is why "parentheses work" is the headline feature.
// - Atoms are resolved by a caller-supplied callback: a profile name expands to its resolved
//   set (recursively, cycle-guarded by the caller); anything else is a literal package token.
//
// The evaluator preserves order: union appends new items in first-seen order; intersection
// keeps the left operand's order; difference keeps the left operand minus the right members.
// This keeps `shall profile show` output stable and readable.

use std::collections::HashSet;

/// Function keywords that take a parenthesized, comma-separated operand list.
const FUNCS: [&str; 3] = ["union", "intersect", "diff"];

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    LParen,
    RParen,
    Comma,
    Pipe,      // |  union
    Amp,       // &  intersect
    Backslash, // \  difference
}

/// A parsed set expression.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Atom(String),
    Union(Box<Expr>, Box<Expr>),
    Intersect(Box<Expr>, Box<Expr>),
    Diff(Box<Expr>, Box<Expr>),
}

/// Returns true if a line looks like a set expression (as opposed to a plain package spec or
/// a legacy `include`/`exclude`/`-pkg` directive). Used by the profile composer to decide
/// whether to hand a line to this evaluator. An expression is anything containing a grouping
/// paren or one of the set operators, or beginning with a function keyword call.
pub fn looks_like_expression(line: &str) -> bool {
    let t = line.trim();
    if t.contains('(') || t.contains('|') || t.contains('&') || t.contains('\\') {
        return true;
    }
    // `union foo` without parens is NOT an expression (it's just an atom list we don't
    // support); a function call always has a paren, already caught above.
    false
}

/// Tokenize a set-expression string. Idents are maximal runs of package/profile characters:
/// letters, digits, and `._/:+@-` (so `g++`, `apt:jq`, `github:o/r`, `node@18` all tokenize
/// as single atoms). Whitespace separates tokens; the set operators and grouping punctuation
/// are single characters.
fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ws if ws.is_whitespace() => {
                chars.next();
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            '|' => {
                chars.next();
                tokens.push(Token::Pipe);
            }
            '&' => {
                chars.next();
                tokens.push(Token::Amp);
            }
            '\\' => {
                chars.next();
                tokens.push(Token::Backslash);
            }
            _ if is_atom_char(c) => {
                let mut s = String::new();
                while let Some(&nc) = chars.peek() {
                    if is_atom_char(nc) {
                        s.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(s));
            }
            other => return Err(format!("unexpected character '{}' in expression", other)),
        }
    }
    Ok(tokens)
}

fn is_atom_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '.' | '_' | '/' | ':' | '+' | '@' | '-')
}

/// Recursive-descent parser over the token stream.
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, want: &Token) -> Result<(), String> {
        match self.next() {
            Some(ref t) if t == want => Ok(()),
            other => Err(format!("expected {:?}, found {:?}", want, other)),
        }
    }

    /// expr := term ( ('|' | '\') term )*   — union / difference, left-associative, lowest.
    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Pipe) => {
                    self.next();
                    let right = self.parse_term()?;
                    left = Expr::Union(Box::new(left), Box::new(right));
                }
                Some(Token::Backslash) => {
                    self.next();
                    let right = self.parse_term()?;
                    left = Expr::Diff(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// term := primary ( '&' primary )*   — intersection, binds tighter than union/diff.
    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;
        while matches!(self.peek(), Some(Token::Amp)) {
            self.next();
            let right = self.parse_primary()?;
            left = Expr::Intersect(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// primary := '(' expr ')' | func '(' expr (',' expr)* ')' | atom
    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::LParen) => {
                let e = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(e)
            }
            Some(Token::Ident(name)) => {
                // A function keyword immediately followed by `(` is a call; otherwise the
                // identifier is a plain atom (which may happen to be spelled like a keyword).
                if FUNCS.contains(&name.as_str()) && matches!(self.peek(), Some(Token::LParen)) {
                    self.parse_function(&name)
                } else {
                    Ok(Expr::Atom(name))
                }
            }
            other => Err(format!("expected an operand, found {:?}", other)),
        }
    }

    fn parse_function(&mut self, name: &str) -> Result<Expr, String> {
        self.expect(&Token::LParen)?;
        let mut args = vec![self.parse_expr()?];
        while matches!(self.peek(), Some(Token::Comma)) {
            self.next();
            args.push(self.parse_expr()?);
        }
        self.expect(&Token::RParen)?;
        if args.is_empty() {
            return Err(format!("{}() needs at least one operand", name));
        }
        // Fold the argument list with the function's binary operator, left-to-right.
        let mut it = args.into_iter();
        let mut acc = it.next().unwrap();
        for a in it {
            acc = match name {
                "union" => Expr::Union(Box::new(acc), Box::new(a)),
                "intersect" => Expr::Intersect(Box::new(acc), Box::new(a)),
                "diff" => Expr::Diff(Box::new(acc), Box::new(a)),
                _ => unreachable!("caller guaranteed a known function keyword"),
            };
        }
        Ok(acc)
    }
}

/// Evaluate a parsed expression to an ordered, de-duplicated package list. `resolve_atom`
/// maps an atom (profile name or package token) to a set: a profile expands to its resolved
/// packages; a bare package resolves to itself.
fn eval(expr: &Expr, resolve_atom: &mut dyn FnMut(&str) -> Vec<String>) -> Vec<String> {
    match expr {
        Expr::Atom(a) => dedup(resolve_atom(a)),
        Expr::Union(l, r) => {
            let mut out = eval(l, resolve_atom);
            let mut seen: HashSet<String> = out.iter().cloned().collect();
            for item in eval(r, resolve_atom) {
                if seen.insert(item.clone()) {
                    out.push(item);
                }
            }
            out
        }
        Expr::Intersect(l, r) => {
            let left = eval(l, resolve_atom);
            let right: HashSet<String> = eval(r, resolve_atom).into_iter().collect();
            left.into_iter().filter(|x| right.contains(x)).collect()
        }
        Expr::Diff(l, r) => {
            let left = eval(l, resolve_atom);
            let right: HashSet<String> = eval(r, resolve_atom).into_iter().collect();
            left.into_iter().filter(|x| !right.contains(x)).collect()
        }
    }
}

fn dedup(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|x| seen.insert(x.clone()))
        .collect()
}

/// Parse and evaluate a set expression. `resolve_atom` is called for every atom; return the
/// atom's resolved package set (for a profile) or a single-element vec (for a bare package).
pub fn evaluate(
    input: &str,
    resolve_atom: &mut dyn FnMut(&str) -> Vec<String>,
) -> Result<Vec<String>, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Ok(vec![]);
    }
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;
    if parser.pos != parser.tokens.len() {
        return Err(format!(
            "trailing tokens after a complete expression (near token {})",
            parser.pos
        ));
    }
    Ok(eval(&expr, resolve_atom))
}
