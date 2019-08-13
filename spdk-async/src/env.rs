use crate::generated::{spdk_dma_free, spdk_dma_malloc};
use libc::c_void;
use std::ptr;

pub struct Buf {
    pub(crate) len: usize,
    pub(crate) ptr: *mut c_void,
}

impl PartialEq for Buf {
    fn eq(&self, other: &Buf) -> bool {
        if self.len != other.len {
            return false;
        }

        let ret = unsafe { libc::memcmp(self.ptr, other.ptr, self.len) };

        ret == 0
    }
}

impl Buf {
    pub fn fill(&mut self, b: i8) {
        unsafe {
            libc::memset(self.ptr, self.len as i32, b as usize);
        }
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        unsafe { spdk_dma_free(self.ptr) }
    }
}

pub fn dma_malloc(size: u64, align: usize) -> Buf {
    let ptr = unsafe { spdk_dma_malloc(size as usize, align, ptr::null_mut() as *mut u64) };
    assert!(!ptr.is_null(), "Failed to malloc");
    Buf {
        len: size as usize,
        ptr,
    }
}
