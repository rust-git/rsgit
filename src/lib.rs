mod attribution;
pub use attribution::Attribution;

mod file_mode;
pub use file_mode::FileMode;

mod git_path;
pub use git_path::CheckPlatforms;
pub use git_path::GitPath;
pub use git_path::GitPathError;
pub use git_path::GitPathSegment;

pub mod object;

pub mod on_disk_repo;

mod path_mode;
pub use path_mode::PathMode;

pub(crate) mod test_support;
