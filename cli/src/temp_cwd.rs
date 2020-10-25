use std::{
    env,
    path::{Path, PathBuf},
};

// A TempCwd allows you to temporarily change the current
// working directory for the host process.
//
// When the struct goes out of scope, the current working
// directory will be reset to its previous value.
//
// Because this struct is intended for testing, its functions
// panic instead of returning Result structs.
//
// We override the dead_code warnings here because this
// struct is only used in test code.
//
// Any test that uses this module should be marked #[serial].
pub(crate) struct TempCwd {
    old_path: PathBuf,
}

impl TempCwd {
    // Temporarily change working directory. The existing working
    // directory will be restored when the struct is dropped.
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(path: P) -> TempCwd {
        let old_path = env::current_dir().unwrap();
        env::set_current_dir(path).unwrap();

        TempCwd { old_path }
    }
}

impl Drop for TempCwd {
    fn drop(&mut self) {
        env::set_current_dir(&self.old_path).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::TempCwd;

    #[test]
    fn temp_cwd() {
        let old_path = env::current_dir().unwrap();
        let tempdir = tempfile::tempdir().unwrap();
        let new_path = tempdir.path();

        {
            let _tcwd = TempCwd::new(new_path);
            assert_ne!(env::current_dir().unwrap(), old_path);
            // MacOS likes to rewrite the path to add a /private
            // prefix, which makes it impossible to assert_eq!(..., new_path) here.
        }

        assert_eq!(env::current_dir().unwrap(), old_path);
    }
}
