use std::env;
use std::io;

#[cfg(windows)]
pub fn set_autostart(enable: bool) -> io::Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hKey: usize,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut usize,
        ) -> i32;
        fn RegSetValueExW(
            hKey: usize,
            lpValueName: *const u16,
            Reserved: u32,
            dwType: u32,
            lpData: *const u8,
            cbData: u32,
        ) -> i32;
        fn RegDeleteValueW(hKey: usize, lpValueName: *const u16) -> i32;
        fn RegCloseKey(hKey: usize) -> i32;
    }

    const HKEY_CURRENT_USER: usize = 0x8000_0001;
    const KEY_SET_VALUE: u32 = 0x0002;
    const REG_SZ: u32 = 1;

    let subkey: Vec<u16> = OsStr::new("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let value_name: Vec<u16> = OsStr::new("Mikey")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut key_handle = 0usize;
    let res = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut key_handle,
        )
    };

    if res != 0 {
        return Err(io::Error::from_raw_os_error(res));
    }

    let ret = if enable {
        let exe_path = env::current_exe()?;
        // --autostart keeps the flyout closed at login; a launch by hand opens it.
        let path_str = format!("\"{}\" --autostart", exe_path.to_string_lossy());
        let wide_path: Vec<u16> = OsStr::new(&path_str)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            RegSetValueExW(
                key_handle,
                value_name.as_ptr(),
                0,
                REG_SZ,
                wide_path.as_ptr() as *const u8,
                (wide_path.len() * 2) as u32,
            )
        }
    } else {
        unsafe { RegDeleteValueW(key_handle, value_name.as_ptr()) }
    };

    unsafe { RegCloseKey(key_handle) };

    if ret == 0 || (!enable && ret == 2) {
        // ERROR_FILE_NOT_FOUND (2) on delete is fine
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(ret))
    }
}

#[cfg(not(windows))]
pub fn set_autostart(enable: bool) -> io::Result<()> {
    if let Ok(home) = env::var("HOME") {
        let dir = std::path::PathBuf::from(home)
            .join(".config")
            .join("autostart");
        let desktop_file = dir.join("mikey.desktop");

        if enable {
            std::fs::create_dir_all(&dir)?;
            let exe_path = env::current_exe()?;
            let content = format!(
                "[Desktop Entry]\nType=Application\nName=Mikey\nExec={}\nTerminal=false\n",
                exe_path.to_string_lossy()
            );
            std::fs::write(&desktop_file, content)?;
        } else if desktop_file.exists() {
            std::fs::remove_file(&desktop_file)?;
        }
    }
    Ok(())
}

#[cfg(windows)]
pub fn is_autostart_enabled() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hKey: usize,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut usize,
        ) -> i32;
        fn RegQueryValueExW(
            hKey: usize,
            lpValueName: *const u16,
            lpReserved: *const u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> i32;
        fn RegCloseKey(hKey: usize) -> i32;
    }

    const HKEY_CURRENT_USER: usize = 0x8000_0001;
    const KEY_QUERY_VALUE: u32 = 0x0001;

    let subkey: Vec<u16> = OsStr::new("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let value_name: Vec<u16> = OsStr::new("Mikey")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut key_handle = 0usize;
    if unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key_handle,
        )
    } != 0
    {
        return false;
    }

    let mut data_len = 0u32;
    let res = unsafe {
        RegQueryValueExW(
            key_handle,
            value_name.as_ptr(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut data_len,
        )
    };
    unsafe { RegCloseKey(key_handle) };
    res == 0
}

#[cfg(not(windows))]
pub fn is_autostart_enabled() -> bool {
    if let Ok(home) = env::var("HOME") {
        std::path::PathBuf::from(home)
            .join(".config")
            .join("autostart")
            .join("mikey.desktop")
            .exists()
    } else {
        false
    }
}
