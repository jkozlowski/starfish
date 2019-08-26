use crate::commitlog::SegmentId;
use lazy_static::lazy_static;
use regex::Regex;
use simple_error::bail;
use simple_error::require_with;
use simple_error::try_with;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

pub enum Version {
    V1,
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

pub struct Descriptor {
    segment_id: SegmentId,
    filename: String,
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
}

mod test {}
