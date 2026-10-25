pub use shall::backends::artifact::platform::*;
use std::path::Path;

fn linux64() -> Platform {
    Platform::new("linux", "x86_64")
}
mod a_foreign_arch_is_rejected;
mod a_foreign_os_is_rejected;
mod a_matching_triple_is_accepted;
mod a_universal_macos_asset_naming_both_arches_is_not_foreign;
mod arm_does_not_match_inside_armv7;
mod silence_on_both_axes_is_accepted;
mod specificity_ranks_an_explicit_target_above_a_silent_one;
mod the_libc_axis_prefers_and_refuses_in_one_direction_only;
mod the_right_os_does_not_excuse_the_wrong_arch;
mod win_does_not_match_inside_a_longer_word;
mod x86_64_is_not_confused_by_the_32_bit_row;
mod x86_does_not_match_inside_x86_64;
