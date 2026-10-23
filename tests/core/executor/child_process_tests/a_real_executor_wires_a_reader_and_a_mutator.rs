use super::*;

/// The two layers a `CommandExecutor` builds must be the two policies, not one policy
/// twice — routing reads through the mutating layer is how the parsers were starved.
///
/// The mutating layer shares stdin wherever `sudo` can be inserted, and only there: on
/// Windows `run_on` never inserts it, so the terminal buys nothing and costs the whole
/// idle bound the first time a manager asks a question (Q35).
#[test]
fn a_real_executor_wires_a_reader_and_a_mutator() {
    let e = CommandExecutor::new(false, false);
    assert!(!e.reader.shares_stdin(), "a read took the terminal");
    assert_eq!(
        e.inner.shares_stdin(),
        !cfg!(windows),
        "sudo must be able to ask for a password on the platforms it is inserted on, and \
         nothing may hold the terminal on the one where it is not"
    );
}
