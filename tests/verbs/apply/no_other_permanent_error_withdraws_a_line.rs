use super::*;

/// Every other variant that `Error::retryability()` also calls `Permanent`. None of them
/// says the name was wrong, and withdrawing on any of them would delete a declaration the
/// user still means — the reason this reads a property and not `retryability()`.
#[test]
fn no_other_permanent_error_withdraws_a_line() {
    let others = [
        Error::Refused("the guard said no".into()),
        Error::Cancelled,
        Error::Config("modules/web.txt:3: bad line".into()),
        Error::Validation("nope".into()),
        Error::Permission("need root".into()),
        Error::BackendNotFound("nosuch".into()),
        Error::Unsupported("purge".into()),
        Error::UnsupportedPlatform("aix".into()),
        Error::Differences("2 changes".into()),
        Error::LuaScript("boom".into()),
        Error::Toml("bad".into()),
        Error::Json("bad".into()),
    ];
    for e in others {
        let label = format!("{e:?}");
        assert_eq!(
            Retryability::Permanent,
            e.retryability(),
            "{label} is not the case this test guards"
        );
        assert!(
            !says_a_name_is_absent(&boxed(e)),
            "{label} would have withdrawn a line the user still means"
        );
    }
}
