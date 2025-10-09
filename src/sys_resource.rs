// ----- Imports ----- //

use std::{
    collections::HashMap,
    env,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

// ----- Structures ----- //

#[derive(Clone)]
pub struct CachedFile {
    pub bytes: Vec<u8>
}

pub struct CachedLoader {
    cache: Arc<Mutex<HashMap<String, CachedFile>>>,
    root_dir: PathBuf,
}

// ----- Implementations ----- //

impl CachedLoader {

    // Initialize a new CachedLoader with a specified base directory
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        let exe_path = env::current_exe().expect("Failed to get current executable path");
        let exe_dir = exe_path.parent().expect("Executable should have a parent directory");
        let root_dir = exe_dir.join(base_dir.as_ref());

        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            root_dir,
        }
    }

    // Load a file from the cache or filesystem
    pub fn load(&self, filename: &str) -> Option<CachedFile> {
        if let Some(cached) = self.cache.lock().unwrap().get(filename).cloned() {
            return Some(cached);
        }

        let path = self.root_dir.join(filename);

        match fs::read(&path) {
            Ok(bytes) => {
                let cached_file = CachedFile { bytes: bytes.clone() };
                self.cache
                    .lock()
                    .unwrap()
                    .insert(filename.to_string(), cached_file.clone());
                println!("Cached file: {:?}", path);
                Some(cached_file)
            }
            Err(_) => None,
        }
    }
}

