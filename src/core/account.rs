use crate::core::{Error, Result};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    name: String,
    home: PathBuf,
    identity: Identity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Identity {
    #[cfg(unix)]
    Unix { uid: u32, gid: u32 },
    #[cfg(windows)]
    Windows { sid: Vec<u32> },
}

impl Account {
    pub fn resolve(name: &str) -> Result<Self> {
        if name.is_empty() {
            return Err(Error::Other("an account name cannot be empty".into()));
        }
        resolve_named(name)
    }

    pub fn current() -> Result<Self> {
        current_account()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn expand_home(&self, value: &str) -> Result<PathBuf> {
        validate_home(&self.home)?;
        if value == "~" {
            Ok(self.home.clone())
        } else if let Some(rest) = value.strip_prefix("~/") {
            Ok(self.home.join(rest))
        } else {
            Ok(PathBuf::from(value))
        }
    }

    pub fn owns(&self, path: &Path, follow: bool) -> Result<bool> {
        owns_path(path, &self.identity, follow)
    }

    pub fn apply_owner(&self, path: &Path, follow: bool) -> Result<()> {
        apply_owner_path(path, &self.identity, follow)
    }

    pub fn bin_dir(&self) -> Result<PathBuf> {
        validate_home(&self.home)?;
        Ok(self.home.join(".local").join("bin"))
    }

    pub fn data_dir(&self) -> Result<PathBuf> {
        validate_home(&self.home)?;
        Ok(data_dir_under(&self.home))
    }

    pub fn launch_agents_dir(&self) -> Result<PathBuf> {
        validate_home(&self.home)?;
        Ok(self.home.join("Library").join("LaunchAgents"))
    }

    pub fn systemd_user_dir(&self) -> Result<PathBuf> {
        validate_home(&self.home)?;
        Ok(self.home.join(".config").join("systemd").join("user"))
    }

    pub fn session_env(&self) -> Vec<(String, String)> {
        let home = self.home.display().to_string();
        let name = self.name.clone();
        let mut env: Vec<(String, String)> = vec![
            ("HOME".to_string(), home.clone()),
            ("USER".to_string(), name.clone()),
            ("LOGNAME".to_string(), name.clone()),
        ];
        #[cfg(unix)]
        {
            let Identity::Unix { uid, .. } = self.identity;
            let runtime = format!("/run/user/{uid}");
            env.push(("XDG_RUNTIME_DIR".to_string(), runtime.clone()));
            env.push((
                "XDG_CONFIG_HOME".to_string(),
                self.home.join(".config").display().to_string(),
            ));
            env.push((
                "XDG_DATA_HOME".to_string(),
                self.home.join(".local").join("share").display().to_string(),
            ));
            env.push((
                "XDG_CACHE_HOME".to_string(),
                self.home.join(".cache").display().to_string(),
            ));
            env.push((
                "DBUS_SESSION_BUS_ADDRESS".to_string(),
                format!("unix:path={runtime}/bus"),
            ));
        }
        #[cfg(windows)]
        {
            env.push(("USERPROFILE".to_string(), home.clone()));
            env.push(("USERNAME".to_string(), name));
            env.push((
                "APPDATA".to_string(),
                self.home
                    .join("AppData")
                    .join("Roaming")
                    .display()
                    .to_string(),
            ));
            env.push((
                "LOCALAPPDATA".to_string(),
                self.home
                    .join("AppData")
                    .join("Local")
                    .display()
                    .to_string(),
            ));
        }
        env
    }

    pub fn is_this_process(&self) -> bool {
        match &self.identity {
            #[cfg(unix)]
            Identity::Unix { uid, .. } => *uid == unsafe { libc::geteuid() },
            #[cfg(windows)]
            Identity::Windows { .. } => current_name()
                .map(|name| name.eq_ignore_ascii_case(&self.name))
                .unwrap_or(false),
        }
    }

    pub fn sid(&self) -> Option<String> {
        // **The Windows half lives in its own function, and that is about the lint as well as
        // the shape.** A `return` inside the `#[cfg(windows)]` block is the only way to write
        // this with the block inline, and `clippy::needless_return` rates it fatal — on the
        // target where it is the *only* body, which is the target nothing lints. A named
        // function has a tail expression like any other and needs no exception.
        #[cfg(windows)]
        {
            self.windows_sid()
        }
        #[cfg(not(windows))]
        {
            None
        }
    }

    #[cfg(windows)]
    fn windows_sid(&self) -> Option<String> {
        let Identity::Windows { sid } = &self.identity;
        sid_text(sid)
    }
}

#[cfg(windows)]
fn sid_text(sid: &[u32]) -> Option<String> {
    use winapi::shared::sddl::ConvertSidToStringSidW;
    use winapi::um::winbase::LocalFree;
    let mut text: *mut u16 = std::ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(sid.as_ptr().cast_mut().cast(), &mut text) } == 0 {
        return None;
    }
    let out = wide_to_string(text)?;
    unsafe { LocalFree(text.cast()) };
    Some(out)
}

/// Where a named account's data lives, by platform convention.
///
/// **macOS is not `~/.local/share`, and the difference is a real path on a real machine.** The
/// account's data directory is what `dirs::data_dir()` answers for the invoking user, so an
/// account that is handed the Linux answer gets a second, shall-only tree that nothing else on
/// the machine knows about — while an *unscoped* schedule still logs where it always logged. Two
/// schedules, two logs, and a reader with no way to tell which is which.
///
/// **The platform is an argument rather than a `cfg` at each arm**, so the convention every
/// platform uses is assertable from any host. A `#[cfg(target_os = "macos")]` test runs on
/// exactly the machine whose bug this was and nowhere else, which is a gate that reports nothing
/// until the day it is the only gate left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Unix,
    MacOs,
    Windows,
}

pub fn host_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOs
    } else if cfg!(windows) {
        Platform::Windows
    } else {
        Platform::Unix
    }
}

pub fn data_dir_under(home: &Path) -> PathBuf {
    match host_platform() {
        Platform::MacOs => home
            .join("Library")
            .join("Application Support")
            .join("shall"),
        Platform::Unix => home.join(".local").join("share").join("shall"),
        Platform::Windows => home.join("AppData").join("Local").join("shall"),
    }
}

fn validate_home(home: &Path) -> Result<()> {
    if home.as_os_str().is_empty() || !home.is_absolute() {
        return Err(Error::Other(format!(
            "account home `{}` is not an absolute non-empty path",
            home.display()
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn current_name() -> Result<String> {
    for key in ["USER", "USERNAME"] {
        if let Ok(name) = std::env::var(key) {
            if !name.is_empty() {
                return Ok(name);
            }
        }
    }
    Err(Error::Other(
        "the current account name is not available from USER or USERNAME".into(),
    ))
}

#[cfg(unix)]
fn resolve_named(name: &str) -> Result<Account> {
    use std::ffi::{CStr, CString};
    use std::os::unix::ffi::OsStringExt;

    let user = CString::new(name)
        .map_err(|_| Error::Other(format!("account name `{}` contains a null byte", name)))?;
    let mut passwd = unsafe { std::mem::zeroed::<libc::passwd>() };
    let mut result: *mut libc::passwd = std::ptr::null_mut();
    let mut buffer = vec![0 as libc::c_char; 4096];
    loop {
        let rc = unsafe {
            libc::getpwnam_r(
                user.as_ptr(),
                &mut passwd,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut result,
            )
        };
        if rc == 0 {
            if result.is_null() {
                return Err(Error::Other(format!("account `{}` was not found", name)));
            }
            let home = unsafe { CStr::from_ptr(passwd.pw_dir) }.to_bytes().to_vec();
            let account = Account {
                name: unsafe { CStr::from_ptr(passwd.pw_name) }
                    .to_string_lossy()
                    .into_owned(),
                home: PathBuf::from(std::ffi::OsString::from_vec(home)),
                identity: Identity::Unix {
                    uid: passwd.pw_uid,
                    gid: passwd.pw_gid,
                },
            };
            validate_home(account.home())?;
            return Ok(account);
        }
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ERANGE) {
            buffer.resize(buffer.len().saturating_mul(2), 0);
            if buffer.len() > 1024 * 1024 {
                return Err(Error::Other(format!(
                    "passwd entry for `{}` is too large: {}",
                    name, error
                )));
            }
            continue;
        }
        return Err(Error::Other(format!(
            "could not resolve account `{}`: {}",
            name, error
        )));
    }
}

#[cfg(unix)]
fn current_account() -> Result<Account> {
    use std::os::unix::ffi::OsStringExt;
    let uid = unsafe { libc::geteuid() };
    let mut passwd = unsafe { std::mem::zeroed::<libc::passwd>() };
    let mut result: *mut libc::passwd = std::ptr::null_mut();
    let mut buffer = vec![0 as libc::c_char; 4096];
    loop {
        let rc = unsafe {
            libc::getpwuid_r(
                uid,
                &mut passwd,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut result,
            )
        };
        if rc == 0 {
            if result.is_null() {
                return Err(Error::Other(format!(
                    "the current account has no passwd entry (uid {})",
                    uid
                )));
            }
            let account = Account {
                name: unsafe { std::ffi::CStr::from_ptr(passwd.pw_name) }
                    .to_string_lossy()
                    .into_owned(),
                home: PathBuf::from(std::ffi::OsString::from_vec(
                    unsafe { std::ffi::CStr::from_ptr(passwd.pw_dir) }
                        .to_bytes()
                        .to_vec(),
                )),
                identity: Identity::Unix {
                    uid: passwd.pw_uid,
                    gid: passwd.pw_gid,
                },
            };
            validate_home(account.home())?;
            return Ok(account);
        }
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ERANGE) {
            buffer.resize(buffer.len().saturating_mul(2), 0);
            if buffer.len() > 1024 * 1024 {
                return Err(Error::Other(format!(
                    "the current passwd entry is too large: {}",
                    error
                )));
            }
            continue;
        }
        return Err(Error::Other(format!(
            "could not resolve the current account: {}",
            error
        )));
    }
}

#[cfg(unix)]
fn owns_path(path: &Path, identity: &Identity, follow: bool) -> Result<bool> {
    use std::os::unix::fs::MetadataExt;
    let Identity::Unix { uid, gid } = *identity;
    let metadata = if follow {
        std::fs::metadata(path)
    } else {
        std::fs::symlink_metadata(path)
    }
    .map_err(Error::from)?;
    Ok(metadata.uid() == uid && metadata.gid() == gid)
}

#[cfg(unix)]
fn apply_owner_path(path: &Path, identity: &Identity, follow: bool) -> Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let Identity::Unix { uid, gid } = *identity;
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| Error::Other(format!("path {:?} contains a null byte", path)))?;
    let rc = if follow {
        unsafe { libc::chown(path.as_ptr(), uid, gid) }
    } else {
        unsafe { libc::lchown(path.as_ptr(), uid, gid) }
    };
    if rc != 0 {
        return Err(Error::Other(format!(
            "could not change ownership of {:?}: {}",
            path,
            std::io::Error::last_os_error()
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn resolve_named(name: &str) -> Result<Account> {
    use winapi::shared::minwindef::{DWORD, FALSE, HKEY};
    use winapi::shared::sddl::ConvertSidToStringSidW;
    use winapi::shared::winerror::ERROR_INSUFFICIENT_BUFFER;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::processenv::ExpandEnvironmentStringsW;
    use winapi::um::winbase::{LocalFree, LookupAccountNameW};
    use winapi::um::winnt::{KEY_READ, KEY_WOW64_64KEY, REG_EXPAND_SZ};
    use winapi::um::winreg::{RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE};

    let wide = wide_name(name)?;
    let mut sid_size: DWORD = 0;
    let mut domain_size: DWORD = 0;
    // **`peUse` is not optional, and passing null here was a STATUS_ACCESS_VIOLATION.** This
    // parameter is a plain out-parameter that the API writes on success — it is not one of the
    // `_opt_` ones, and nothing in the call's type says so. With two null buffers the first
    // call fails before reaching the write, which is why the first lookup survived and the
    // second — the one that succeeds — killed the process (#101).
    let mut pe_use: DWORD = 0;
    let first = unsafe {
        LookupAccountNameW(
            std::ptr::null_mut(),
            wide.as_ptr(),
            std::ptr::null_mut(),
            &mut sid_size,
            std::ptr::null_mut(),
            &mut domain_size,
            &mut pe_use,
        )
    };
    // The only acceptable outcome of a lookup with no buffers: it fails, and it says why.
    // The condition used to be written the other way round — `first != FALSE && …` — so a lookup
    // that had already failed for an unrelated reason (an unknown account is `ERROR_NONE_MAPPED`)
    // was treated as a sized request and carried on with a zero-length SID.
    if first != FALSE || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER {
        return Err(Error::Other(format!(
            "could not look up account `{}`: {}",
            name,
            std::io::Error::last_os_error()
        )));
    }
    let sid_words = (sid_size as usize)
        .checked_add(std::mem::size_of::<u32>() - 1)
        .ok_or_else(|| Error::Other("the account SID length overflowed".into()))?
        / std::mem::size_of::<u32>();
    let mut sid = vec![0u32; sid_words];
    let mut domain = vec![0u16; domain_size as usize];
    let mut sid_capacity: DWORD = sid
        .len()
        .checked_mul(std::mem::size_of::<u32>())
        .and_then(|length| DWORD::try_from(length).ok())
        .ok_or_else(|| Error::Other("the account SID length overflowed".into()))?;
    let rc = unsafe {
        LookupAccountNameW(
            std::ptr::null_mut(),
            wide.as_ptr(),
            sid.as_mut_ptr().cast(),
            &mut sid_capacity,
            domain.as_mut_ptr(),
            &mut domain_size,
            &mut pe_use,
        )
    };
    if rc == FALSE {
        return Err(Error::Other(format!(
            "could not look up account `{}`: {}",
            name,
            std::io::Error::last_os_error()
        )));
    }
    let mut sid_text: *mut u16 = std::ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(sid.as_ptr().cast_mut().cast(), &mut sid_text) } == FALSE {
        return Err(Error::Other(format!(
            "could not format the identity for `{}`",
            name
        )));
    }
    // **A wide string, read as a narrow one.** `ConvertSidToStringSidW` returns `LPWSTR`, and
    // `CStr::from_ptr` on that buffer stops at the first `0x00` byte — which is the high half of
    // the first `u16`. The SID therefore read back as `"S"`, the `ProfileList` key below did not
    // exist, and `Account::current()` failed on Windows for every account. That is #106's
    // missing drift row: a `link:` target resolves through the current account.
    let sid_key = wide_to_string(sid_text).ok_or_else(|| {
        Error::Other(format!(
            "could not read the identity of `{name}` back from Windows"
        ))
    })?;
    unsafe { LocalFree(sid_text.cast()) };

    let key = wide_string(&format!(
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList\\{}",
        sid_key
    ))?;
    let mut hkey: HKEY = std::ptr::null_mut();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            key.as_ptr(),
            0,
            KEY_READ | KEY_WOW64_64KEY,
            &mut hkey,
        )
    };
    if status != 0 {
        return Err(Error::Other(format!(
            "could not find a home directory for account `{}` (registry error {})",
            name, status
        )));
    }
    let value = wide_string("ProfileImagePath")?;
    let mut bytes = vec![0u8; 32768];
    let mut size = bytes.len() as DWORD;
    let mut kind = 0;
    let status = unsafe {
        RegQueryValueExW(
            hkey,
            value.as_ptr(),
            std::ptr::null_mut(),
            &mut kind,
            bytes.as_mut_ptr().cast(),
            &mut size,
        )
    };
    unsafe { RegCloseKey(hkey) };
    if status != 0 || kind != REG_EXPAND_SZ {
        return Err(Error::Other(format!(
            "could not read the home directory for account `{}` (registry error {})",
            name, status
        )));
    }
    let size = usize::try_from(size)
        .ok()
        .filter(|size| *size <= bytes.len())
        .ok_or_else(|| Error::Other("the home directory value length overflowed".into()))?;
    bytes.truncate(size);
    if !bytes.len().is_multiple_of(std::mem::size_of::<u16>()) {
        return Err(Error::Other(format!(
            "the home for account `{}` is not valid UTF-16",
            name
        )));
    }
    let mut raw = bytes
        .chunks_exact(std::mem::size_of::<u16>())
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    while raw.last() == Some(&0) {
        raw.pop();
    }
    raw.push(0);
    let required = unsafe { ExpandEnvironmentStringsW(raw.as_ptr(), std::ptr::null_mut(), 0) };
    if required == 0 {
        return Err(Error::Other(format!(
            "could not expand the home directory for account `{}`",
            name
        )));
    }
    let mut expanded = vec![0u16; required as usize];
    let length =
        unsafe { ExpandEnvironmentStringsW(raw.as_ptr(), expanded.as_mut_ptr(), required) };
    if length == 0 || length > required {
        return Err(Error::Other(format!(
            "could not expand the home directory for account `{}`",
            name
        )));
    }
    expanded.truncate(length as usize);
    while expanded.last() == Some(&0) {
        expanded.pop();
    }
    let home = PathBuf::from(
        String::from_utf16(&expanded)
            .map_err(|_| Error::Other(format!("the home for account `{}` is not UTF-16", name)))?,
    );
    let account = Account {
        name: name.to_string(),
        home,
        identity: Identity::Windows { sid },
    };
    validate_home(account.home())?;
    Ok(account)
}

#[cfg(windows)]
fn current_account() -> Result<Account> {
    current_name().and_then(|name| resolve_named(&name))
}

#[cfg(windows)]
fn owns_path(path: &Path, identity: &Identity, follow: bool) -> Result<bool> {
    use winapi::um::accctrl::SE_FILE_OBJECT;
    use winapi::um::aclapi::{GetNamedSecurityInfoW, GetSecurityInfo};
    use winapi::um::securitybaseapi::EqualSid;
    use winapi::um::winbase::LocalFree;
    use winapi::um::winnt::OWNER_SECURITY_INFORMATION;
    // Irrefutable, and deliberately written as one. `Identity` carries exactly one variant per
    // platform, so on Windows this pattern always matches and the `else` was unreachable —
    // a warning, which CI rates fatal. It also ran *backwards*: the arm that never fires was the
    // one that said "return false", so a third variant added later would have been answered with
    // a wrong answer rather than a compile error. The unix side at line 88 already takes the
    // identity this way.
    let Identity::Windows { sid } = identity;
    let path_wide = wide_path(path)?;
    let mut owner = std::ptr::null_mut();
    let mut descriptor = std::ptr::null_mut();
    let status = if follow {
        unsafe {
            GetNamedSecurityInfoW(
                path_wide.as_ptr(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION,
                &mut owner,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut descriptor,
            )
        }
    } else {
        let handle = open_windows_path(path, false)?;
        let status = unsafe {
            GetSecurityInfo(
                handle,
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION,
                &mut owner,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut descriptor,
            )
        };
        unsafe { winapi::um::handleapi::CloseHandle(handle) };
        status
    };
    if status != 0 {
        return Err(Error::Other(format!(
            "could not read ownership of {:?}: {}",
            path, status
        )));
    }
    let equal = unsafe { EqualSid(owner, sid.as_ptr().cast_mut().cast()) } != 0;
    unsafe { LocalFree(descriptor.cast()) };
    Ok(equal)
}

#[cfg(windows)]
fn apply_owner_path(path: &Path, identity: &Identity, follow: bool) -> Result<()> {
    use winapi::um::accctrl::SE_FILE_OBJECT;
    use winapi::um::aclapi::{SetNamedSecurityInfoW, SetSecurityInfo};
    use winapi::um::winnt::OWNER_SECURITY_INFORMATION;
    // Irrefutable for the same reason as `owns_path` above.
    let Identity::Windows { sid } = identity;
    let path_wide = wide_path(path)?;
    let status = if follow {
        unsafe {
            SetNamedSecurityInfoW(
                // winapi types the name as LPWSTR, which is `*mut u16`, so a shared `Vec<u16>`
                // has to say so rather than let inference pick the `*const` reading.
                path_wide.as_ptr().cast_mut(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION,
                sid.as_ptr().cast_mut().cast(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                // The `Dacl` argument. A null DACL alongside flags that name only the OWNER is
                // how Win32 is told to leave the DACL alone, which is what the two arguments
                // above it already asked for — the call was written with six arguments because
                // this one is invisible in the C declaration's own documentation, and the
                // function has seven.
                std::ptr::null_mut(),
            )
        }
    } else {
        let handle = open_windows_path(path, true)?;
        let status = unsafe {
            SetSecurityInfo(
                handle,
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION,
                sid.as_ptr().cast_mut().cast(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        unsafe { winapi::um::handleapi::CloseHandle(handle) };
        status
    };
    if status != 0 {
        return Err(Error::Other(format!(
            "could not change ownership of {:?}: {}",
            path, status
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn open_windows_path(path: &Path, write: bool) -> Result<winapi::um::winnt::HANDLE> {
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::winbase::{FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT};
    use winapi::um::winnt::{
        FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, READ_CONTROL, WRITE_OWNER,
    };
    let path = wide_path(path)?;
    let access = if write { WRITE_OWNER } else { READ_CONTROL };
    let handle = unsafe {
        CreateFileW(
            path.as_ptr(),
            access,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            // `lpSecurityAttributes` is `PSECURITY_ATTRIBUTES` — a mutable pointer even when it
            // is null, because Win32 may write through it. `std::ptr::null()` infers `*const _`
            // from the parameter and does not compile; `null_mut()` states the null.
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS,
            std::ptr::null_mut(),
        )
    };
    if handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
        return Err(Error::Other(format!(
            "could not open {:?} for ownership: {}",
            path,
            std::io::Error::last_os_error()
        )));
    }
    Ok(handle)
}

/// A NUL-terminated UTF-16 buffer, as Win32 returns them, read as text.
///
/// One reader for both SID strings, because the bug was one reader written twice: `sid_text`
/// and `resolve_named` each decoded a wide buffer with `CStr::from_ptr` and each produced `"S"`.
#[cfg(windows)]
fn wide_to_string(wide: *const u16) -> Option<String> {
    if wide.is_null() {
        return None;
    }
    let mut len = 0usize;
    while unsafe { *wide.add(len) } != 0 {
        len += 1;
    }
    Some(String::from_utf16_lossy(unsafe {
        std::slice::from_raw_parts(wide, len)
    }))
}

#[cfg(windows)]
fn wide_name(name: &str) -> Result<Vec<u16>> {
    Ok(name.encode_utf16().chain(std::iter::once(0)).collect())
}

#[cfg(windows)]
fn wide_string(value: &str) -> Result<Vec<u16>> {
    wide_name(value)
}

#[cfg(windows)]
fn wide_path(path: &Path) -> Result<Vec<u16>> {
    use std::os::windows::ffi::OsStrExt;
    Ok(path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect())
}

#[cfg(not(any(unix, windows)))]
fn resolve_named(name: &str) -> Result<Account> {
    Err(Error::Other(format!(
        "account lookup is unavailable for `{}`",
        name
    )))
}

#[cfg(not(any(unix, windows)))]
fn current_account() -> Result<Account> {
    resolve_named("current")
}
