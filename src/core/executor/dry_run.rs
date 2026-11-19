//! The layer a preview runs on: it records what would have run and answers nothing.

use super::*;

pub struct DryRunExecutor {
    vfs: Arc<DashMap<PathBuf, String>>,
}

impl DryRunExecutor {
    pub fn new(vfs: Arc<DashMap<PathBuf, String>>) -> Self {
        Self { vfs }
    }
}

#[async_trait]
impl ExecutionLayer for DryRunExecutor {
    async fn execute(
        &self,
        cmd: &str,
        args: &[String],
        _env: &HashMap<String, String>,
    ) -> Result<StdOutput> {
        crate::would!("Would execute: {} {}", cmd, args.join(" "));
        Ok(DryRunOutput::new().into())
    }

    /// Whether a command exists is a fact about this machine, not something a preview gets
    /// to invent. Answering `true` for everything made every backend look installed.
    fn check_command(&self, cmd: &str) -> bool {
        RawExecutor::reader().check_command(cmd)
    }

    async fn symlink(&self, src: &Path, dst: &Path) -> Result<()> {
        let val = format!("LINK:{}", src.display());
        self.vfs.insert(dst.to_path_buf(), val);
        Ok(())
    }
}
