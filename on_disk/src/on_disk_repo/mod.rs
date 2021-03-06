use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use flate2::{write::ZlibEncoder, Compression};

use rsgit_core::{
    object::Object,
    repo::{Error, Repo, Result},
};

/// Implementation of [`Repo`] that stores content on the local file system.
///
/// _IMPORTANT NOTE:_ This is intended as a reference implementation largely
/// for testing purposes and may not necessarily handle all of the edge cases that
/// the traditional `git` command-line interface will handle.
///
/// That said, it does intentionally use the same `.git` folder format as command-line
/// `git` so that results may be compared for similar operations.
///
/// [`Repo`]: ../rsgit_core/repo/trait.Repo.html
#[derive(Debug)]
pub struct OnDiskRepo {
    work_dir: PathBuf,
    git_dir: PathBuf,
}

impl OnDiskRepo {
    /// Create an on-disk git repository.
    ///
    /// `work_dir` should be the top-level working directory. A `.git` directory should
    /// exist at this path. Use [`init`] function to create an empty on-disk repository if
    /// necessary.
    ///
    /// [`init`]: #method.init
    pub fn new<P: AsRef<Path>>(work_dir: P) -> Result<Self> {
        let work_dir = work_dir.as_ref().to_path_buf();
        if !work_dir.exists() {
            return Err(Error::WorkDirDoesntExist(work_dir));
        }

        let git_dir = work_dir.join(".git");
        if !git_dir.exists() {
            return Err(Error::GitDirDoesntExist(git_dir));
        }

        Ok(OnDiskRepo { work_dir, git_dir })
    }

    /// Creates a new, empty git repository on the local file system.
    ///
    /// Analogous to [`git init`].
    ///
    /// [`git init`]: https://git-scm.com/docs/git-init
    pub fn init<P: AsRef<Path>>(work_dir: P) -> Result<Self> {
        let git_dir = work_dir.as_ref().join(".git");
        if git_dir.exists() {
            return Err(Error::GitDirShouldntExist(git_dir));
        }

        fs::create_dir_all(&git_dir)?;

        create_config(&git_dir)?;
        create_description(&git_dir)?;
        create_head(&git_dir)?;
        create_hooks_dir(&git_dir)?;
        create_info_dir(&git_dir)?;
        create_objects_dir(&git_dir)?;
        create_refs_dir(&git_dir)?;

        Ok(OnDiskRepo {
            work_dir: work_dir.as_ref().to_path_buf(),
            git_dir,
        })
    }

    /// Return the working directory for this repo.
    pub fn work_dir(&self) -> &Path {
        self.work_dir.as_path()
    }

    /// Return the path to the `.git` directory.
    pub fn git_dir(&self) -> &Path {
        self.git_dir.as_path()
    }
}

impl Repo for OnDiskRepo {
    fn put_loose_object(&mut self, object: &Object) -> Result<()> {
        let object_id = object.id().to_string();
        let (dir, path) = object_id.split_at(2);

        let mut object_path = self.git_dir.join("objects");
        object_path.push(dir);
        fs::create_dir(&object_path)?;

        object_path.push(path);
        write_object_to_path(object, object_path.as_ref())
    }
}

// --- init helpers ---

fn create_config(git_dir: &Path) -> Result<()> {
    let config_path = git_dir.join("config");
    let config_txt = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n";

    fs::write(config_path, config_txt).map_err(|e| e.into())
}

fn create_description(git_dir: &Path) -> Result<()> {
    let desc_path = git_dir.join("description");
    let desc_txt = "Unnamed repository; edit this file 'description' to name the repository.\n";

    fs::write(desc_path, desc_txt).map_err(|e| e.into())
}

fn create_head(git_dir: &Path) -> Result<()> {
    let head_path = git_dir.join("HEAD");
    let head_txt = "ref: refs/heads/master\n";

    fs::write(head_path, head_txt).map_err(|e| e.into())
}

fn create_hooks_dir(git_dir: &Path) -> Result<()> {
    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).map_err(|e| e.into())

    // NOTE: Intentionally not including the sample files.
}

fn create_info_dir(git_dir: &Path) -> Result<()> {
    let info_dir = git_dir.join("info");
    fs::create_dir_all(&info_dir)?;

    let exclude_path = info_dir.join("exclude");
    let exclude_txt = "# git ls-files --others --exclude-from=.git/info/exclude\n# Lines that start with '#' are comments.\n# For a project mostly in C, the following would be a good set of\n# exclude patterns (uncomment them if you want to use them):\n# *.[oa]\n# *~\n.DS_Store\n";

    fs::write(exclude_path, exclude_txt).map_err(|e| e.into())
}

fn create_objects_dir(git_dir: &Path) -> Result<()> {
    let info_dir = git_dir.join("objects/info");
    fs::create_dir_all(&info_dir)?;

    let pack_dir = git_dir.join("objects/pack");
    fs::create_dir_all(&pack_dir).map_err(|e| e.into())
}

fn create_refs_dir(git_dir: &Path) -> Result<()> {
    let heads_dir = git_dir.join("refs/heads");
    fs::create_dir_all(&heads_dir)?;

    let tags_dir = git_dir.join("refs/tags");
    fs::create_dir_all(&tags_dir).map_err(|e| e.into())
}

// --- put_loose_object helpers ---

fn write_object_to_path(object: &Object, path: &Path) -> Result<()> {
    let file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let mut z = ZlibEncoder::new(file, Compression::new(1));

    let header = format!("{} {}\0", object.kind(), object.len()).into_bytes();
    z.write_all(&header)?;

    let mut object_reader = object.open()?;
    io::copy(&mut object_reader, &mut z)?;

    z.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests;
