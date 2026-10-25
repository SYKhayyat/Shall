use super::*;

#[tokio::test]
async fn a_tool_that_cannot_answer_is_not_guessed_at() {
    let (exec, cfg) = mocked(&[]);
    for be in ["yarn", "pnpm", "bun", "npm", "opam"] {
        assert!(
            user_bin_dirs(be, &cfg, &exec).await.is_empty(),
            "{be} did not answer and something answered for it"
        );
        assert!(unreachable_warning(be, &cfg, &exec).await.is_none());
    }
}
