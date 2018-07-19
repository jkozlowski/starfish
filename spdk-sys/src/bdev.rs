use failure::Error;
use generated::spdk_bdev_bindings::{
    spdk_bdev_get_by_name, spdk_bdev   
};
use std::ffi::CString;
use std::os::raw::{ c_char, c_void };
use std::ptr;

#[derive(Debug, Fail)]
pub enum BDevError {
    #[fail(display = "Could not find a bdev: {}", _0)]
    NotFound(String),
}

/// SPDK block device.
#[derive(Debug)]
pub struct BDev {
    name: String,
    bdev: *mut spdk_bdev
}

pub fn get_by_name<S>(name: S) -> Result<BDev, Error>
where S: Into<String> + Clone {
    let name_cstring = CString::new(name.clone().into())
            .expect("Couldn't create a string");
            
    let bdev = unsafe { spdk_bdev_get_by_name(name_cstring.as_ptr()) };
    if bdev.is_null() {
        return Err(BDevError::NotFound(name.clone().into()))?;
    }

    return Ok(BDev {name: name.into(), bdev });
}
