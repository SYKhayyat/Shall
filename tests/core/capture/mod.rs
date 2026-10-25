pub use shall::core::capture::*;
pub use shall::core::capture::{Capped, MAX_CAPTURED_BYTES};
mod a_capture_past_the_ceiling_keeps_the_head_and_the_tail;
mod a_capture_under_the_ceiling_is_byte_for_byte_what_was_written;
