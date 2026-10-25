pub use shall::core::prompt::*;
pub use shall::core::{Error, Result};
use std::io::IsTerminal;
mod a_declining_prompt_with_nobody_there_answers_no;
mod a_refusing_prompt_with_nobody_there_refuses_by_name;
mod an_attended_confirm_still_lets_yes_answer_first;
mod yes_answers_without_asking_anything;
