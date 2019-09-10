use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use futures::TryStreamExt;
use regex::Regex;
use simple_error::bail;
use simple_error::require_with;
use simple_error::try_with;
use slog;

use lazy_static::lazy_static;

use crate::commitlog::SegmentId;
use crate::fs::FileSystem;

pub enum Version {
    V1,
}

impl Into<u32> for Version {
    fn into(self) -> u32 {
        match self {
            Version::V1 => 1
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Version::V1 => write!(f, "1"),
        }
    }
}

impl TryFrom<&str> for Version {
    type Error = Box<dyn std::error::Error>;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let value = try_with!(u8::from_str(s), "Failed to parse version");

        if value != 1 {
            bail!("Only V1 supported: {}", value)
        } else {
            Ok(Version::V1)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Descriptor {
    segment_id: SegmentId,
    filename: String,
}

impl slog::KV for Descriptor {
    fn serialize(
        &self,
        _rec: &slog::Record<'_>,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_u64("segment_it", self.segment_id)?;
        serializer.emit_str("filename", &self.filename)
    }
}

impl Descriptor {
    pub fn create(segment_id: SegmentId) -> Self {
        // CommitLog-1-1234.log
        let filename = format!("CommitLog-{}-{}.log", Version::V1, segment_id);
        Descriptor {
            segment_id,
            filename,
        }
    }

    pub fn try_create<T: AsRef<str>>(file_name: T) -> Result<Descriptor, Box<dyn Error>> {
        let file_name_ref = file_name.as_ref();

        lazy_static! {
            static ref REGEX: Regex = Regex::new(r"^CommitLog\-(\d+)\-(\d+)\.log$").unwrap();
        }

        let caps = require_with!(
            REGEX.captures(file_name_ref),
            "File name does not match the format: {}",
            file_name_ref
        );

        let _version = Version::try_from(&caps[1])?;
        let segment_id = SegmentId::from_str(&caps[2])?;
        Ok(Descriptor::create(segment_id))
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn version(&self) -> Version {
        Version::V1
    }

    pub fn segment_id(&self) -> SegmentId {
        self.segment_id
    }

    pub async fn list_descriptors<P>(
        fs: &mut FileSystem,
        path: P,
    ) -> Result<Vec<Descriptor>, Box<dyn Error>>
        where
            P: AsRef<Path> + Clone + Send + 'static,
    {
        let mut files = fs.read_dir(path.clone()).await?;

        let mut descriptors: Vec<Descriptor> = vec![];

        while let Some(elem) = files.try_next().await? {
            let os_file_name = elem.file_name();
            let file_name = os_file_name.to_str().expect("Failed to get file name");
            let descriptor = Descriptor::try_create(file_name)?;
            descriptors.push(descriptor);
        }

        Ok(descriptors)
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use futures::channel::oneshot;
    use hamcrest2::assert_that;
    use hamcrest2::contains;
    use hamcrest2::prelude::*;
    use tempdir::TempDir;
    use tokio::runtime::Builder;

    use crate::fs::FileSystem;

    use super::*;

    #[test]
    fn test_list_descriptors() -> Result<(), String> {
        let rt = Builder::new().build().unwrap();

        let (sender, receiver) = oneshot::channel::<Result<(), String>>();

        rt.spawn(async move {
            let res: Result<(), Box<dyn Error>> = (async {
                let tmp_dir = TempDir::new("test_list_descriptors")?;

                create_file(&tmp_dir, "CommitLog-1-13.log")?;
                create_file(&tmp_dir, "CommitLog-1-1234.log")?;

                let mut fs = FileSystem::create().await?;

                let tmp_dir_tmp = tmp_dir.path().to_owned();
                let descriptors = Descriptor::list_descriptors(&mut fs, tmp_dir_tmp).await?;

                assert_that!(
                    &descriptors,
                    contains(vec!(Descriptor::create(13), Descriptor::create(1234))).exactly()
                );
                Ok(())
            })
                .await;

            sender.send(res.map_err(|_| "Oops".to_owned()));
        });

        futures::executor::block_on(receiver).unwrap()
    }

    fn create_file(tmp_dir: &TempDir, file_name: &str) -> std::io::Result<()> {
        let file_path = tmp_dir.path().join(file_name);
        File::create(file_path).map(|_| ())
    }
}
