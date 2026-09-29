#[cfg(windows)]
#[test]
fn test_softcam_dll_registration() {
    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryW(lpLibFileName: *const u16) -> usize;
        fn GetProcAddress(
            hModule: usize,
            lpProcName: *const std::ffi::c_char,
        ) -> *const std::ffi::c_void;
        fn FreeLibrary(hModule: usize) -> i32;
    }
    #[link(name = "advapi32")]
    extern "system" {
        fn RegCreateKeyExW(
            hKey: usize,
            lpSubKey: *const u16,
            reserved: u32,
            lpClass: *mut u16,
            dwOptions: u32,
            samDesired: u32,
            lpSecurityAttributes: *mut std::ffi::c_void,
            phkResult: *mut usize,
            lpdwDisposition: *mut u32,
        ) -> i32;
        fn RegOverridePredefKey(hKey: usize, hNewHKey: usize) -> i32;
        fn RegCloseKey(hKey: usize) -> i32;
    }

    const HKEY_CLASSES_ROOT: usize = 0xFFFF_FFFF_8000_0000;
    const HKEY_CURRENT_USER: usize = 0xFFFF_FFFF_8000_0001;
    let subkey: Vec<u16> = "Software\\Classes\0".encode_utf16().collect();
    let mut hkcu_classes = 0usize;

    let res = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            std::ptr::null_mut(),
            0,
            0x2001F,
            std::ptr::null_mut(),
            &mut hkcu_classes,
            std::ptr::null_mut(),
        )
    };
    assert_eq!(res, 0, "RegCreateKeyExW failed");

    let override_res = unsafe { RegOverridePredefKey(HKEY_CLASSES_ROOT, hkcu_classes) };
    println!("RegOverridePredefKey result: {}", override_res);

    let dll_path: Vec<u16> = "softcam.dll\0".encode_utf16().collect();
    let h_module = unsafe { LoadLibraryW(dll_path.as_ptr()) };
    assert_ne!(h_module, 0, "Failed to load softcam.dll");

    let p_reg = unsafe { GetProcAddress(h_module, c"DllRegisterServer".as_ptr()) };
    assert!(!p_reg.is_null(), "DllRegisterServer not found");

    type FnDllRegisterServer = unsafe extern "system" fn() -> i32;
    let reg_fn: FnDllRegisterServer = unsafe { std::mem::transmute(p_reg) };
    let reg_hr = unsafe { reg_fn() };
    println!("DllRegisterServer returned HRESULT: 0x{:08X}", reg_hr);

    unsafe {
        RegOverridePredefKey(HKEY_CLASSES_ROOT, 0);
        RegCloseKey(hkcu_classes);
        FreeLibrary(h_module);
    }

    assert_eq!(reg_hr, 0, "DllRegisterServer should return S_OK (0)");
}

#[cfg(windows)]
#[test]
fn test_directshow_device_enumeration() {
    #[link(name = "ole32")]
    extern "system" {
        fn CoInitialize(pvReserved: *mut std::ffi::c_void) -> i32;
        fn CoUninitialize();
    }

    unsafe { CoInitialize(std::ptr::null_mut()) };

    mikey::video::vcam::install::ensure_directshow_registered(&std::path::PathBuf::from(
        "softcam.dll",
    ));

    let key_path = "Software\\Classes\\CLSID\\{860BB310-5D01-11D0-BD3B-00A0C911CE86}\\Instance\\DirectShow Softcam\0";
    let subkey: Vec<u16> = key_path.encode_utf16().collect();
    const HKEY_CURRENT_USER: usize = 0x8000_0001;
    let mut h_key = 0usize;

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
            lpReserved: *mut u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> i32;
        fn RegCloseKey(hKey: usize) -> i32;
    }

    let open_res =
        unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, 0x20019, &mut h_key) };
    assert_eq!(open_res, 0, "Mikey Cam DirectShow key must be openable");

    let friendly_name_key: Vec<u16> = "FriendlyName\0".encode_utf16().collect();
    let mut val_type = 0u32;
    let mut data = vec![0u8; 256];
    let mut len = 256u32;
    let query_res = unsafe {
        RegQueryValueExW(
            h_key,
            friendly_name_key.as_ptr(),
            std::ptr::null_mut(),
            &mut val_type,
            data.as_mut_ptr(),
            &mut len,
        )
    };
    assert_eq!(query_res, 0, "FriendlyName should be queryable");
    let name = String::from_utf16_lossy(unsafe {
        std::slice::from_raw_parts(data.as_ptr() as *const u16, (len as usize) / 2)
    });
    println!(
        "Enumerated DirectShow capture device FriendlyName: {}",
        name.trim_matches('\0')
    );
    assert_eq!(name.trim_matches('\0'), "Mikey Cam");

    unsafe {
        RegCloseKey(h_key);
        CoUninitialize();
    }
}
