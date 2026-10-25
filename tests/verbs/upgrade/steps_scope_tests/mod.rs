pub use shall::app::sync::resolver::StateResolver;
pub use shall::verbs::perform_maintenance;
pub use shall::verbs::prelude::*;
pub use shall::verbs::setup::handle_canary;
pub use shall::verbs::sync::{enforce_policy, print_flight_plan};
pub use shall::verbs::upgrade::*;

fn req<'a>(
    packages: &'a [String],
    backend: Option<&'a str>,
    security: bool,
    steps: Option<bool>,
) -> UpgradeRequest<'a> {
    const NONE: &Option<String> = &None;
    UpgradeRequest {
        packages,
        backend,
        all: false,
        security,
        except: &[],
        ignore_holds: false,
        ignore_pins: false,
        profile: NONE,
        module: NONE,
        out: Output::Human,
        canary: false,
        test: NONE,
        steps,
    }
}
fn named(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}
mod an_unscoped_upgrade_is_not_narrowed;
mod every_way_of_narrowing_a_run_stops_the_steps;
mod the_flags_reach_both_answers_the_default_does_not_give;
