//! A capture with a ceiling.
//!
//! **Every accumulating path in this tree has a bound except the one that kept the bytes.**
//! `core::download` refuses a body over 2 GiB before it fills the disk, and the terminal mirror
//! caps what it prints — while `RawExecutor::wait_watched`'s `pump` read a child's stdout and
//! stderr into a `Vec` in 8 KiB chunks with no cap of any kind, holding a manager invocation's
//! complete output in memory until the command exited. A `nix` build, a `cargo install`
//! compiling, a progress bar redrawn with carriage returns for an hour: each grows that without
//! limit, and a concurrent wave holds one per running manager.
//!
//! Its own module rather than another hundred lines of `executor.rs`, which is the file
//! `a_module_is_a_subject_not_a_pile` was about — "what a captured stream keeps" is a subject,
//! and it is not the same subject as "how a child is owned and stopped".

/// The most of one child's output Shall keeps in memory, per stream.
///
/// 8 MiB: far past any real diagnostic and far short of a problem. What is kept is the head and
/// the tail, because those are the two parts an error message wants — the invocation that
/// started it and the failure that ended it — and the middle of a three-hour build log is
/// neither.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub const MAX_CAPTURED_BYTES: usize = 8 * 1024 * 1024;

/// A capture that keeps the head and the tail of a stream and drops the middle.
///
/// See [`MAX_CAPTURED_BYTES`]. The truncation is announced in the bytes
/// themselves, because what this becomes is an error message a person reads, and a log
/// with a silent hole in it is worse than one that says where the hole is.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub struct Capped {
    head: Vec<u8>,
    tail: std::collections::VecDeque<u8>,
    dropped: usize,
}

impl Capped {
    /// Half the budget to each end.
    const HALF: usize = MAX_CAPTURED_BYTES / 2;

    pub fn new() -> Self {
        Self {
            head: Vec::new(),
            tail: std::collections::VecDeque::new(),
            dropped: 0,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) {
        let mut bytes = bytes;
        if self.head.len() < Self::HALF {
            let room = Self::HALF - self.head.len();
            let take = room.min(bytes.len());
            self.head.extend_from_slice(&bytes[..take]);
            bytes = &bytes[take..];
        }
        for &b in bytes {
            self.tail.push_back(b);
            if self.tail.len() > Self::HALF {
                self.tail.pop_front();
                self.dropped += 1;
            }
        }
    }

    pub fn finish(self) -> Vec<u8> {
        if self.dropped == 0 {
            let mut out = self.head;
            out.extend(self.tail);
            return out;
        }
        let mut out = self.head;
        out.extend_from_slice(
            format!(
                "\n… [shall: {} bytes of output omitted; the head and tail are kept] …\n",
                self.dropped
            )
            .as_bytes(),
        );
        out.extend(self.tail);
        out
    }
}

impl Default for Capped {
    fn default() -> Self {
        Self::new()
    }
}
