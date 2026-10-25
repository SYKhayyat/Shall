use super::*;

#[test]
fn env_is_stripped_because_the_lookup_happens_here() {
    // `/usr/bin/env python3` names a PATH search, and there is no `/usr/bin/env` on Windows
    // to perform it. Leaving it in place is the difference between finding `python` and
    // reporting that `/usr/bin/env` is missing.
    let found = shebang("#!/usr/bin/env python3\n").expect("a shebang");
    assert_eq!(found.interpreter, "python3");

    let dash_s = shebang("#!/usr/bin/env -S python3 -u\n").expect("a shebang");
    assert_eq!(dash_s.interpreter, "python3");
    assert_eq!(dash_s.args, ["-u"]);
}
