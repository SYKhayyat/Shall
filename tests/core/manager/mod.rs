use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};
pub use shall::core::manager::*;
pub use shall::core::{Package, PackageSpec, Result};
use std::sync::Arc;

struct Catalogue {
    answers: Vec<&'static str>,
    qualifies: bool,
}
#[async_trait]
impl Searchable for Catalogue {
    fn qualifies_names(&self) -> bool {
        self.qualifies
    }
    async fn search(&self, _query: &str) -> Result<Vec<Package>> {
        Ok(self
            .answers
            .iter()
            .map(|n| Package::new(*n, "mock-manager"))
            .collect())
    }
}
fn catalogue(qualifies: bool, answers: &[&'static str]) -> Catalogue {
    Catalogue {
        answers: answers.to_vec(),
        qualifies,
    }
}
mod an_exact_name_beats_an_atom_that_ends_the_same_way;
mod an_exact_name_is_the_answer_and_the_only_one_an_ordinary_manager_gives;
mod more_than_one_atom_is_refused_and_names_them_all;
mod one_matching_atom_resolves_and_answers_with_the_atom;
mod the_unqualified_half_is_everything_after_the_last_slash;
