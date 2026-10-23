pub use shall::core::Package;
pub use shall::parsers::pacman::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;

const PACMAN: &str = "\
archlinux-keyring 20260707.1-1 -> 20260727-1
audit 4.1.4-2 -> 4.2.1-1
gcc 16.1.1+r581+gb73ad535acaa-1 -> 16.1.1+r595+g171d15ac6959-1
";
mod nothing_upgradable_is_nothing;
mod pacman_reads_the_right_side_of_the_arrow;
