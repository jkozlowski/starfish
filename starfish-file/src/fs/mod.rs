mod file;
mod filesystem;

pub use file::File;
pub use filesystem::FileSystem;

#[derive(Debug, Error)]
pub enum Error {
     #[error(display = "Commitlog has been shut down. Cannot add data")]
    Closed,
}
