use super::*;
use shall::core::executor::DryRunOutput;

#[tokio::test]
async fn a_tool_that_cannot_answer_is_not_guessed_at() {
    // **The silence these five tools keep is the subject**, so it is stated rather than fallen
    // through (`#162`): a command that prints nothing has answered nothing, and the assertion is
    // that Shall reports no directory instead of guessing one.
    let (exec, cfg) = mocked(&[
        ("yarn global bin", ""),
        ("pnpm bin -g", ""),
        ("bun pm bin -g", ""),
        ("npm prefix -g", ""),
        ("opam var bin", ""),
    ]);
    for be in ["yarn", "pnpm", "bun", "npm", "opam"] {
        assert!(
            user_bin_dirs(be, &cfg, &exec).await.is_empty(),
            "{be} did not answer and something answered for it"
        );
        assert!(unreachable_warning(be, &cfg, &exec).await.is_none());
    }
}
