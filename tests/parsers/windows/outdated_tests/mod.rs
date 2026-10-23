pub use shall::core::Package;
pub use shall::parsers::windows::*;
pub use shall::parsers::{or_unrecognised, ParseResult, Unrecognised};
pub use shall::utils::text::sanitize;

const WINGET: &str = "\
Name                                                         Id                                    Version                       Available                     Source
---------------------------------------------------------------------------------------------------------------------------------------------------------------------
7-Zip 25.01 (x64)                                            7zip.7zip                             25.01                         26.02                         winget
Adobe Acrobat (64-bit)                                       Adobe.Acrobat.Pro                     25.001.21111                  26.001.21771                  winget
";
const SCOOP: &str = "\
WARN  Scoop bucket(s) out of date. Run 'scoop update' to get the latest changes.
Name    Installed Version Latest Version Missing Dependencies Info
----    ----------------- -------------- -------------------- ----
7zip    26.00             26.02\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20
kubectl 1.36.2            1.36.3\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20
";
mod a_pinned_choco_package_is_not_reported_as_outdated;
mod nothing_outdated_parses_as_nothing_rather_than_a_row;
mod scoop_reports_the_latest_not_the_installed_version;
mod winget_never_reports_the_display_name;
mod winget_reports_the_id_and_the_available_version;
