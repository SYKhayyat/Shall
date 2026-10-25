pub use shall::app::profile_expr::*;
use std::collections::HashMap;
use std::collections::HashSet;

fn resolver(map: HashMap<&'static str, Vec<&'static str>>) -> impl FnMut(&str) -> Vec<String> {
    move |atom: &str| match map.get(atom) {
        Some(v) => v.iter().map(|s| s.to_string()).collect(),
        None => vec![atom.to_string()],
    }
}
fn eval_str(input: &str, map: &[(&'static str, &[&'static str])]) -> Result<Vec<String>, String> {
    let m: HashMap<&'static str, Vec<&'static str>> =
        map.iter().map(|(k, v)| (*k, v.to_vec())).collect();
    let mut r = resolver(m);
    evaluate(input, &mut r)
}
mod bare_atom_resolves_to_itself;
mod difference_removes_right_members;
mod function_forms_match_infix;
mod infinite_nesting_groups;
mod intersect_binds_tighter_than_union;
mod intersect_keeps_only_common_in_left_order;
mod looks_like_expression_detects_operators;
mod package_atoms_with_plus_and_colon_tokenize;
mod parentheses_override_precedence;
mod trailing_junk_errors;
mod unbalanced_parens_error;
mod union_infix_dedups_in_order;
