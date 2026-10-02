//! What Shall put on this machine, in a file that does not expire.
//!
//! **This is a record, not a second opinion about who owns what.** Ownership is the registry,
//! entirely (`Q55`, II.56, `V.186`), and nothing here is ever consulted to decide it. What it
//! answers is a narrower question that the journal was being asked to keep answering and cannot:
//! *did Shall install this?* — which is a fact about what Shall did, and which is the evidence a
//! claim about Shall's own action is allowed to rest on.
//!
//! **It exists because the journal expires and the thing it witnesses does not.**
//! `cleanup_expired_logs(7)` drops a `Completed` entry after seven days, and that was sound for
//! every other reader of it — recovery wants *open* work, and an operation that finished seven
//! days ago has nothing to recover. It was unsound for the one reader that wants the opposite:
//! `reconcile_ownership` reads `Completed` entries as *"Shall put this here"*, and a kill between
//! the per-operation WAL write and the once-per-run registry write leaves a package installed and
//! owned by nobody. Repaired on the next sync, yes — but only if the evidence outlives the window
//! in which somebody gets round to running one. A machine left alone for a week kept its orphan
//! for ever, and the only repair left to it was a listing that reports an unpacked-but-not-
//! configured package as *not installed*, which is the lister being right (`PLAN.md` #105).
//!
//! **So: same key, same writer, one lifetime longer.** Nothing is duplicated — the line is
//! appended by the same call that appends the WAL entry, from the same [`JournalAction`], in the
//! same place — and the file never expires. That is the whole difference, and it is the whole
//! fix.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::core::{Error, Result};

/// One line: Shall installed this from this manager, at this time.
///
/// The timestamp is not read by anything yet, and it is here because a record of what happened
/// with no record of when is half a receipt — and because a file that cannot say when cannot be
/// pruned by age later, which is the property this file is here to have.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Receipt {
    pub backend: String,
    pub name: String,
    pub at_unix: i64,
}

impl Receipt {
    /// The `backend:name` key the registry and the journal both speak.
    pub fn key(&self) -> String {
        format!("{}:{}", self.backend, self.name)
    }
}

/// Shall's receipts, keyed `backend:name`, loaded forward and last-writer-wins.
///
/// The key is the [`Journal`](crate::core::journal::Journal)'s own key so the two readers can be
/// unioned without either translating.
#[derive(Debug, Default)]
pub struct Receipts {
    path: PathBuf,
    entries: BTreeMap<String, Receipt>,
}

impl Receipts {
    /// `.jsonl` beside `journal.jsonl`, like every other one-JSON-value-per-line file here.
    pub const FILE_NAME: &'static str = "receipts.jsonl";

    /// The receipts that belong beside a WAL at `journal_path`.
    ///
    /// A sibling rather than a path of its own so there is no second derivation of "where Shall
    /// keeps its data" to forget — the same reason `Journal::at` is injected rather than
    /// re-derived, which is spelled out at its own doc comment.
    pub fn sibling_of(journal_path: &Path) -> PathBuf {
        journal_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(Self::FILE_NAME)
    }

    /// Open the receipts beside a WAL. An absent file is an empty record, which is the ordinary
    /// state of a machine on which Shall has installed nothing.
    pub fn at(path: PathBuf) -> Result<Self> {
        let mut receipts = Self {
            path,
            entries: BTreeMap::new(),
        };
        receipts.load_sync()?;
        Ok(receipts)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// A missing file is an empty record; an unreadable one is reported, because a receipt file
    /// that cannot be read is a claim about ownership that cannot be made and should not look
    /// like an absence of claims.
    fn load_sync(&mut self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }
        let data = std::fs::read_to_string(&self.path).map_err(|e| {
            Error::Io(format!(
                "Failed to read the install receipts at {:?}: {}",
                self.path, e
            ))
        })?;
        let mut unreadable = 0usize;
        for line in data.lines().filter(|l| !l.trim().is_empty()) {
            match serde_json::from_str::<Receipt>(line) {
                Ok(receipt) => {
                    self.entries.insert(receipt.key(), receipt);
                }
                // **A torn tail is skipped, not fatal, and a wholly unreadable file is not
                // fatal either** — the same rule `Journal::load_sync` follows and for the same
                // reason: this file is evidence about the past, so failing to read it must not
                // fail the command trying to repair the present. `S10` says a corrupt WAL may
                // not brick every command.
                Err(_) => unreadable += 1,
            }
        }
        if unreadable > 0 {
            tracing::warn!(
                "{} line(s) of the install receipts at {:?} could not be read and were skipped; \
                 {} package(s) remain on record. Claims resting on a receipt this run could not \
                 read will be re-taken from a listing instead.",
                unreadable,
                self.path,
                self.entries.len()
            );
        }
        Ok(())
    }

    /// Record that Shall installed `name` from `backend`.
    ///
    /// **Idempotent, and says so by not writing.** A package installed, removed and installed
    /// again is one package on this machine, and a receipt file that grew a line per attempt
    /// would answer "what did Shall put here" with a count of how many times it tried. The
    /// original timestamp is kept, because the question the file is asked is *whether*, not
    /// *when*.
    pub fn record(&mut self, backend: &str, name: &str, at_unix: i64) -> Result<bool> {
        let key = format!("{}:{}", backend, name);
        if self.entries.contains_key(&key) {
            return Ok(false);
        }
        let receipt = Receipt {
            backend: backend.to_string(),
            name: name.to_string(),
            at_unix,
        };
        self.entries.insert(key, receipt.clone());
        self.append(&receipt)?;
        Ok(true)
    }

    /// Append one line. A preview appends nothing, through the same helper the WAL uses — a
    /// dry-run that wrote a receipt would leave behind evidence of an install it did not do, and
    /// evidence is the one thing this file holds.
    fn append(&self, receipt: &Receipt) -> Result<()> {
        let line = serde_json::to_string(receipt)
            .map_err(|e| Error::Other(format!("Failed to serialize an install receipt: {}", e)))?;
        crate::utils::file::append_lines(&self.path, &[line.as_str()])
            .map(|_| ())
            .map_err(|e| Error::Persist(format!("Write of the install receipts failed: {}", e)))
    }

    /// Whether Shall's own record says it put this here.
    pub fn claims(&self, backend: &str, name: &str) -> bool {
        self.entries.contains_key(&format!("{}:{}", backend, name))
    }

    /// Every `backend:name` on record, in a stable order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    /// Every receipt, in a stable order. Keyed `backend:name`, so two readers can be unioned
    /// without either translating.
    pub fn entries(&self) -> impl Iterator<Item = &Receipt> {
        self.entries.values()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn a_receipt_round_trips_through_the_file() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join(Receipts::FILE_NAME);
        let mut written = Receipts::at(path.clone()).unwrap();
        assert!(written.record("cargo", "hexyl", 1).unwrap());

        let read = Receipts::at(path).unwrap();
        assert!(read.claims("cargo", "hexyl"));
        assert!(
            !read.claims("cargo", "fd"),
            "a receipt is about one package"
        );
        assert_eq!(read.len(), 1);
        assert_eq!(read.keys().collect::<Vec<_>>(), ["cargo:hexyl"]);
    }

    /// **The property the file exists for, stated as a test rather than a comment: a receipt
    /// outlives the journal purge.** Written with a timestamp far enough in the past that the
    /// seven-day rule would have dropped its counterpart, and still read back.
    #[test]
    fn a_receipt_is_still_there_a_year_later() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join(Receipts::FILE_NAME);
        let old = 1_600_000_000; // 2020-09-13, well past any age rule.
        let mut written = Receipts::at(path.clone()).unwrap();
        written.record("apt", "jq", old).unwrap();

        let read = Receipts::at(path).unwrap();
        assert!(read.claims("apt", "jq"));
        assert_eq!(read.entries.get("apt:jq").unwrap().at_unix, old);
    }

    /// The claim is *whether*, so a second install of the same package is not a second line.
    /// A file that grew per attempt would answer "what did Shall put here" with a count of tries.
    #[test]
    fn installing_the_same_package_twice_is_one_receipt() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join(Receipts::FILE_NAME);
        let mut receipts = Receipts::at(path.clone()).unwrap();
        assert!(
            receipts.record("apt", "jq", 100).unwrap(),
            "the first is new"
        );
        assert!(
            !receipts.record("apt", "jq", 200).unwrap(),
            "the second says nothing new"
        );

        let body = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            body.lines().filter(|l| !l.trim().is_empty()).count(),
            1,
            "one line on disk, not one per attempt:\n{body}"
        );
        // And the timestamp kept is the first — when it arrived, not when it was re-installed.
        assert_eq!(Receipts::at(path).unwrap().entries["apt:jq"].at_unix, 100);
    }

    /// **A torn tail is not a corrupt file.** A crash partway through an append leaves one
    /// truncated line; every whole line before it is still true. `S10`: this file is evidence
    /// about the past, so failing to read it must not fail the command repairing the present.
    #[test]
    fn a_truncated_last_line_does_not_lose_the_whole_file() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join(Receipts::FILE_NAME);
        let mut receipts = Receipts::at(path.clone()).unwrap();
        receipts.record("apt", "jq", 1).unwrap();
        receipts.record("cargo", "hexyl", 2).unwrap();

        let mut body = std::fs::read_to_string(&path).unwrap();
        body.push_str("{\"backend\":\"npm\",\"name\":\"cows");
        std::fs::write(&path, body).unwrap();

        let read = Receipts::at(path).unwrap();
        assert!(read.claims("apt", "jq"));
        assert!(read.claims("cargo", "hexyl"));
        assert!(!read.claims("npm", "cows"));
    }

    /// An absent file is an empty record, which is the ordinary state of a machine where Shall
    /// has installed nothing — not an error, and not a reason to fail a command.
    #[test]
    fn no_file_is_an_empty_record() {
        let tmp = tempdir().unwrap();
        let receipts = Receipts::at(tmp.path().join("never-written.jsonl")).unwrap();
        assert!(receipts.is_empty());
        assert!(!receipts.claims("apt", "jq"));
    }

    #[test]
    fn the_receipts_file_is_the_wals_sibling() {
        assert_eq!(
            Receipts::sibling_of(Path::new("/data/journal.jsonl")),
            PathBuf::from("/data/receipts.jsonl")
        );
        // A journal path with no directory beside it still lands somewhere writable-relative
        // rather than panicking on `parent()` returning None.
        assert_eq!(
            Receipts::sibling_of(Path::new("journal.jsonl")),
            PathBuf::from("receipts.jsonl")
        );
    }

    #[test]
    fn a_receipts_file_that_cannot_be_read_is_an_error_not_an_absence() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("receipts.jsonl");
        std::fs::create_dir(&dir).unwrap();
        let err = Receipts::at(dir).unwrap_err();
        assert!(
            err.to_string().contains("install receipts"),
            "the error names the file rather than reading as 'no claims': {err}"
        );
    }
}
