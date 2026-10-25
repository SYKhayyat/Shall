use super::*;

/// Real `ufw status numbered` output. Headers and chatter must not become phantom rules —
/// the S22/S23 class, one domain over.
#[test]
fn ufw_status_parses_only_the_rules() {
    let out = "Status: active\n\
               \n\
                    To                         Action      From\n\
                    --                         ------      ----\n\
               [ 1] 22/tcp                     ALLOW IN    Anywhere\n\
               [ 2] 8080/udp                   ALLOW IN    Anywhere\n";
    let rules = shipped("ufw").parse_rules(out);
    assert_eq!(rules.len(), 2, "{:?}", rules);
    assert_eq!(rules[0].to_string(), "22/tcp");
    assert_eq!(rules[1].to_string(), "8080/udp");
}
