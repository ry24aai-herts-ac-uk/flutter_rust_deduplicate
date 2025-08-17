use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json;
use std::ffi::{CStr, CString, c_char};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};
use tokio::runtime::Runtime;

static DB: OnceCell<Surreal<Db>> = OnceCell::new();
static RUNTIME: OnceCell<Runtime> = OnceCell::new();

#[derive(Debug, Serialize, Deserialize, Clone)]
struct HashEntry {
    file_name: String,
    file_path: String,
    blake3_hash: String,
    is_duplicate: bool,
}

fn init_db() -> &'static Surreal<Db> {
    DB.get_or_init(|| {
        let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());
        runtime.block_on(async {
            let db = Surreal::new::<RocksDb>("/home/rajasekhar/photoBase/photobase/flutter_ffi_blake3rust/hashes.db").await.unwrap();
            db.use_ns("hashes_ns").use_db("hashes_db").await.unwrap();
            db.query("DEFINE TABLE hashes;").await.unwrap();
            db.query("DEFINE INDEX file_name_index ON hashes COLUMNS file_name;")
                .await
                .unwrap();
            db.query("DEFINE INDEX blake3_hash_index ON hashes COLUMNS blake3_hash;")
                .await
                .unwrap();
            db
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn blake3_hash_file(path: *const c_char) -> *const c_char {
    let db = init_db();
    let runtime = RUNTIME.get().unwrap();
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
    };

    let file_path = Path::new(path_str);
    let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();

    let mut file = match File::open(path_str) {
        Ok(f) => f,
        Err(_) => return std::ptr::null(),
    };

    let mut buffer = [0; 4096];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return std::ptr::null(),
    };

    let mut full_hasher = blake3::Hasher::new();
    full_hasher.update(&buffer[..bytes_read]);

    if bytes_read == 4096 {
        let mut rest_buffer = [0; 65536];
        loop {
            let bytes_read = match file.read(&mut rest_buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => return std::ptr::null(),
            };
            full_hasher.update(&rest_buffer[..bytes_read]);
        }
    }

    let full_hash = full_hasher.finalize();
    let full_hash_hex = full_hash.to_hex().to_string();

    let entry = runtime.block_on(async {
        let is_duplicate = {
            let result: Option<HashEntry> = db.select(("hashes", &full_hash_hex)).await.unwrap();
            result.is_some()
        };

        let entry = HashEntry {
            file_name: file_name.clone(),
            file_path: path_str.to_string(),
            blake3_hash: full_hash_hex.clone(),
            is_duplicate,
        };

        if !is_duplicate {
            let _: Option<HashEntry> = db
                .create(("hashes", &full_hash_hex))
                .content(entry.clone())
                .await
                .unwrap();
        }

        entry
    });

    let result_json = serde_json::to_string(&entry).unwrap();
    let c_str_result = CString::new(result_json).unwrap();
    c_str_result.into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}