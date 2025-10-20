// ----- Imports ----- //

use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};


// ----- Structures ----- //

#[derive(Clone)]
pub struct CachedFile {
    pub bytes: Vec<u8>,
}

pub struct CachedLoader {
    cache: Arc<Mutex<HashMap<String, CachedFile>>>,
    pub root_dir: PathBuf,
}

// ----- Implementations ----- //

impl CachedLoader {
    // Initialize a new CachedLoader with a specified base directory
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        

        let exe_path = env::current_exe().expect("Failed to get current executable path");
        let exe_dir = exe_path
            .parent()
            .expect("Executable should have a parent directory");
        let root_dir = exe_dir.join(base_dir.as_ref());

        println!(
            "[CachedLoader] Executable dir: {}\n[CachedLoader] Base dir argument: {}\n[CachedLoader] Resolved root_dir: {}",
            exe_dir.display(),
            base_dir.as_ref().display(),
            root_dir.display()
        );

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
        let path = PathBuf::from(self.clean_path(path.to_str().unwrap_or("")));

        match fs::read(&path) {
            Ok(bytes) => {
                let cached_file = CachedFile {
                    bytes: bytes.clone(),
                };
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

    fn clean_path(&self, path: &str) -> String {
        let mut cleaned = path.replace("..", "");
        if cleaned.starts_with('/') {
            cleaned.remove(0);
        }
        // Replace backslashes with forward slashes for cross-platform compatibility
        cleaned = cleaned.replace('\\', "/");
        cleaned
    }
}
