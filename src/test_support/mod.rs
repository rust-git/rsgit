mod temp_git_repo;

// TempGitRepo is only used in test code. Override the unused warning.
#[allow(unused_imports)]
pub(crate) use temp_git_repo::TempGitRepo;
