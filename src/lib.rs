mod attribution;
pub use attribution::Attribution;

mod content_source;
pub use content_source::ContentSource;

mod object;
pub use object::Object;
pub use object::ObjectKind;
pub use object::ParseObjectIdError;
pub use object::ParseObjectIdErrorKind;

pub mod on_disk_repo;

pub(crate) mod test_support;
