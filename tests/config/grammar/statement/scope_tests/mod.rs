use super::*;

pub use shall::config::grammar::statement::*;
fn o() -> Origin {
    Origin::new("modules/dev.txt", 3)
}
fn known(name: &str) -> bool {
    matches!(name, "apt" | "cargo")
}
fn pv(line: &str) -> Result<Statement> {
    let s = parse(&o(), line, &known)?;
    validate(&o(), &s)?;
    Ok(s)
}

mod a_misspelled_scope_is_refused_and_lists_the_legal_ones;
mod scope_is_accepted_on_the_three_statements_that_can_vary;
mod scope_is_refused_where_it_means_nothing;
mod writing_the_default_scope_is_not_an_error;
