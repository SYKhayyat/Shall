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
        #[cfg(windows)]
        {
            let Identity::Windows { sid } = &self.identity;
            return sid_text(sid);
        }
        #[cfg(not(windows))]
        None
    }
}

#[cfg(windows)]
fn sid_text(sid: &[u32]) -> Option<String> {
    use winapi::shared::sddl::ConvertSidToStringSidW;
    use winapi::um::winbase::LocalFree;
    let mut text: *mut u16 = std::ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(sid.as_ptr().cast(), &mut text) } == 0 {
        return None;
    }
    let out = unsafe { std::ffi::CStr::from_ptr(text.cast()) }
        .to_string_lossy()
        .into_owned();
    unsafe { LocalFree(text.cast()) };
    Some(out)
}

#[cfg(not(windows))]
fn data_dir_under(home: &Path) -> PathBuf {
    home.join(".local").join("share").join("shall")
}

#[cfg(windows)]
fn data_dir_under(home: &Path) -> PathBuf {
    home.join("AppData").join("Local").join("shall")
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
    let first = unsafe {
        LookupAccountNameW(
            std::ptr::null_mut(),
            wide.as_ptr(),
            std::ptr::null_mut(),
            &mut sid_size,
            std::ptr::null_mut(),
            &mut domain_size,
        )
    };
    if first != FALSE && unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER {
        return Err(Error::Other(format!(
            "could not look up account `{}`",
            name
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
        )
    };
    if rc == FALSE {
        return Err(Error::Other(format!(
            "could not look up account `{}`",
            name
        )));
    }
    let mut sid_text: *mut u16 = std::ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(sid.as_ptr().cast(), &mut sid_text) } == FALSE {
        return Err(Error::Other(format!(
            "could not format the identity for `{}`",
            name
        )));
    }
    let sid_key = unsafe { std::ffi::CStr::from_ptr(sid_text.cast()) }
        .to_string_lossy()
        .into_owned();
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
    if bytes.len() % std::mem::size_of::<u16>() != 0 {
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
    let Identity::Windows { sid } = identity else {
        return Ok(false);
    };
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
    let equal = unsafe { EqualSid(owner, sid.as_ptr().cast()) } != 0;
    unsafe { LocalFree(descriptor.cast()) };
    Ok(equal)
}

#[cfg(windows)]
fn apply_owner_path(path: &Path, identity: &Identity, follow: bool) -> Result<()> {
    use winapi::um::accctrl::SE_FILE_OBJECT;
    use winapi::um::aclapi::{SetNamedSecurityInfoW, SetSecurityInfo};
    use winapi::um::winnt::OWNER_SECURITY_INFORMATION;
    let Identity::Windows { sid } = identity else {
        return Ok(());
    };
    let path_wide = wide_path(path)?;
    let status = if follow {
        unsafe {
            SetNamedSecurityInfoW(
                path_wide.as_ptr(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION,
                sid.as_ptr().cast(),
                std::ptr::null_mut(),
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
                sid.as_ptr().cast(),
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
    use winapi::um::fileapi::CreateFileW;
    use winapi::um::winbase::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, OPEN_EXISTING,
    };
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
            std::ptr::null(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_account_has_a_home_and_an_identity() {
        let account = Account::current().unwrap();
        assert!(!account.name().is_empty());
        assert!(!account.home().as_os_str().is_empty());
    }

    #[test]
    fn unknown_accounts_are_refused() {
        assert!(Account::resolve("shall-no-such-account-71").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn the_session_bus_names_the_account_asked_for_and_not_the_one_running() {
        for name in ["nobody", "daemon", "bin", "root"] {
            let Ok(account) = Account::resolve(name) else {
                continue;
            };
            let env: std::collections::HashMap<String, String> =
                account.session_env().into_iter().collect();
            let runtime = env
                .get("XDG_RUNTIME_DIR")
                .unwrap_or_else(|| panic!("{name} has no runtime directory"));
            let bus = env.get("DBUS_SESSION_BUS_ADDRESS").unwrap_or_else(|| {
                panic!("{name} inherited a session bus or was left without one")
            });
            assert_eq!(bus, &format!("unix:path={runtime}/bus"), "for {name}");
        }
    }
}
