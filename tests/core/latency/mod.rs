pub use shall::core::latency::*;
use std::fmt::Write;
pub use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::Subscriber;

fn captured<F: FnOnce()>(f: F) -> Vec<String> {
    struct Collect(Arc<Mutex<Vec<String>>>);
    impl Subscriber for Collect {
        fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
            true
        }
        fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }
        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
        fn event(&self, event: &tracing::Event<'_>) {
            let mut visitor = MessageVisitor(Arc::new(Mutex::new(String::new())));
            event.record(&mut visitor);
            let text = { std::mem::take(&mut *visitor.0.lock().unwrap()) };
            self.0.lock().unwrap().push(text);
        }
        fn enter(&self, _: &tracing::span::Id) {}
        fn exit(&self, _: &tracing::span::Id) {}
    }
    struct MessageVisitor(Arc<Mutex<String>>);
    impl tracing::field::Visit for MessageVisitor {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            let _ = write!(self.0.lock().unwrap(), "{} ", field.name());
            let _ = write!(self.0.lock().unwrap(), "{value:?}");
        }
    }
    let seen = Arc::new(Mutex::new(Vec::new()));
    let guard = tracing::subscriber::set_default(Collect(Arc::clone(&seen)));
    f();
    // Dropped before the read: the guard is what makes this thread's subscriber the default,
    // and holding it across the return would borrow `seen` for the rest of the function.
    drop(guard);
    let out = seen.lock().unwrap().clone();
    out
}
fn markers(lines: &[String]) -> Vec<&str> {
    lines
        .iter()
        .filter(|l| l.contains(UNMEASURED_TOKEN))
        .map(|l| l.trim())
        .collect()
}
mod a_balanced_host_is_measured_however_long_it_took;
mod a_command_inside_its_budget_is_not_reported;
mod a_serialised_fan_out_is_out_of_budget_and_a_real_one_is_not;
mod a_serialised_plan_is_out_of_budget_and_a_deep_one_is_not;
mod a_skewed_host_cannot_measure_the_ratio_but_can_still_measure_collapse;
mod a_skewed_run_that_took_more_than_one_wave_is_still_judged;
mod only_the_fan_out_class_carries_a_shape;
mod the_marker_carries_a_stable_greppable_token;
mod the_two_classes_shall_controls_carry_a_budget_and_the_others_do_not;
mod too_few_children_is_not_a_measurement;
mod unmeasured_is_reported_on_every_skip_path;

mod a_fan_out_of_managers_is_asked_concurrently_tests;
mod latency_budget_tests;
mod the_other_fan_outs_fan_out_too_tests;
