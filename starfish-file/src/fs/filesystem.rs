use crate::fs::File;
use crate::fs::Error;
use std::fs::File as StdFile;
use std::fs::OpenOptions;
use std::path::Path;

pub struct FileSystem {

}

impl FileSystem {

    pub async fn open<P: AsRef<Path>>(&self, path: P, options: OpenOptions) -> Result<File, Error> {
        unimplemented!();
    } 
}