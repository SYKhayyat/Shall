//! What "which package owns this file" reads, one file per manager dialect (`#211`).
//!
//! Four managers answer four different shapes, and three of them answer with the **version
//! attached** — `openssh-server-9.9p1-8.fc41.x86_64` — which no declaration on this tree can
//! carry. That is the whole reason these readers exist rather than a `split_whitespace().next()`.

pub use shall::parsers::{apt, common, dnf, pacman};

mod a_diverted_file_names_the_package_that_diverted_it;
mod a_versioned_answer_is_read_as_a_name;
