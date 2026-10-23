pub use shall::core::Package;
pub use shall::parsers::windows::*;
pub use shall::parsers::{or_unrecognised, ParseResult, Unrecognised};
pub use shall::utils::text::sanitize;

const EXPORT: &str = r#"{
    "$schema": "https://aka.ms/winget-packages.schema.2.0.json",
    "CreationDate": "2026-08-05T12:19:44.000-00:00",
    "Sources": [
        {
            "Packages": [
                { "PackageIdentifier": "7zip.7zip" },
                { "PackageIdentifier": "Git.Git" },
                { "PackageIdentifier": "Microsoft.WindowsAppRuntime.1.8" },
                { "PackageIdentifier": "Microsoft.WindowsAppRuntime.1.8" },
                { "PackageIdentifier": "Notepad++.Notepad++" }
            ],
            "SourceDetails": { "Name": "winget" }
        }
    ],
    "WinGetVersion": "1.12.360"
}"#;
mod a_blank_identifier_is_not_a_package;
mod a_package_exported_once_per_architecture_is_declared_once;
mod an_empty_export_is_an_empty_machine_and_a_broken_one_is_not;
mod an_export_yields_its_package_identifiers;
mod no_pseudo_identifier_can_reach_a_declaration_through_the_export;
