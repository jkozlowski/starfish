use failure::Error;
use std::ptr;

use crate::bdev::BDev;
use crate::generated;
use crate::generated::spdk_blob_bdev_bindings::{spdk_bdev_create_bs_dev, spdk_bs_dev};

#[derive(Debug, Fail)]
pub enum BlobBDevError {
    #[fail(display = "Could not create blob bdev!: {}", _0)]
    FailedToCreate(String),
}

/// SPDK blob store block device.
///
/// This is a virtual representation of a block device that is exported by the backend.
/// TODO: Implement Drop
#[derive(Debug)]
pub struct BlobStoreBDev {
    pub(crate) bs_dev: *mut spdk_bs_dev,
}

pub fn create_bs_dev(bdev: &mut BDev) -> Result<BlobStoreBDev, Error> {
    let bs_dev = unsafe {
        spdk_bdev_create_bs_dev(
            // PITA that bindgen seems to generate the mappings multiple times...
            bdev.bdev as *mut generated::spdk_blob_bdev_bindings::spdk_bdev,
            None,
            ptr::null_mut(),
        )
    };

    if bs_dev.is_null() {
        return Err(BlobBDevError::FailedToCreate(bdev.name.clone().into()))?;
    }

    return Ok(BlobStoreBDev { bs_dev });
}
