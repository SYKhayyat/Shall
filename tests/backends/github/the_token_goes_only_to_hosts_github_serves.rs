use super::*;

#[test]
fn the_token_goes_only_to_hosts_github_serves() {
    for good in [
        "https://api.github.com/repos/x/y/releases/latest",
        "https://github.com/owner/repo/releases/download/v1/a.zip",
        "https://www.github.com/owner/repo/releases/download/v1/a.zip",
    ] {
        assert!(
            GithubBackendCore::token_belongs_here(good),
            "GitHub's own host must still be tokened: {good}"
        );
    }
    for bad in [
        "https://objects.githubusercontent.com/secrets",
        "https://evil.example.com/payload",
        "https://api.github.com.evil.example.com/repos/x/y",
        "not a url at all",
    ] {
        assert!(
            !GithubBackendCore::token_belongs_here(bad),
            "the bearer must not ride to {bad}"
        );
    }
}
