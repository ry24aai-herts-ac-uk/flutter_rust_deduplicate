use std::ffi::{CStr, c_char};
use std::fs::File;
use std::io::Read;

#[unsafe(no_mangle)]
pub extern "C" fn blake3_hash_file(path: *const c_char) -> *const c_char {
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
    };

    let mut file = match File::open(path_str) {
        Ok(f) => f,
        Err(_) => return std::ptr::null(),
    };

    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; 65536];

    loop {
        let bytes_read = match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => return std::ptr::null(),
        };
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    let hash_hex = hash.to_hex().to_string();

    // Leak the string to keep it alive
    let c_str_hash = std::ffi::CString::new(hash_hex).unwrap();
    let ptr = c_str_hash.as_ptr();
    std::mem::forget(c_str_hash);
    ptr
}
