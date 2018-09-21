use crate::generated::{spdk_bdev, spdk_bdev_get_by_name};
use failure::Error;
use std::ffi::CString;

#[derive(Debug, Fail)]
pub enum BDevError {
    #[fail(display = "Could not find a bdev: {}", _0)]
    NotFound(String),
}

/// SPDK block device.
/// TODO: Implement Drop
#[derive(Debug)]
pub struct BDev {
    pub(crate) name: String,
    pub(crate) bdev: *mut spdk_bdev,
}

pub fn get_by_name<S>(name: S) -> Result<BDev, Error>
where
    S: Into<String> + Clone,
{
    let name_cstring = CString::new(name.clone().into()).expect("Couldn't create a string");

    let bdev = unsafe { spdk_bdev_get_by_name(name_cstring.as_ptr()) };
    if bdev.is_null() {
        return Err(BDevError::NotFound(name.clone().into()))?;
    }

    Ok(BDev {
        name: name.into(),
        bdev,
    })
}
