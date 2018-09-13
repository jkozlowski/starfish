use crate::generated::spdk_env_bindings::{
    spdk_dma_malloc,
    spdk_dma_free
};
use std::os::raw::{c_void};
use std::ptr;
use libc;

pub struct Buf {
    pub (crate) len: usize,
    pub (crate) ptr: *mut c_void
}

impl Buf {
    pub fn fill(&mut self, b: i8) {
        unsafe {
            libc::memset(
                self.ptr as *mut libc::c_void, 
                self.len as i32, 
                b as usize
            );
        }
    }

    pub fn cmp(&self, other: &Buf) -> bool {
        if self.len != other.len {
            return false;
        }
        
        let ret = unsafe {
            libc::memcmp(self.ptr as *const libc::c_void,
                   other.ptr as *const libc::c_void, 
                   self.len)
        };

        return !(ret != 0);
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        unsafe { 
            spdk_dma_free(self.ptr) 
        }
    }
}

pub fn dma_malloc(size: u64, align: usize) -> Buf {
    let ptr = unsafe {
        spdk_dma_malloc(
            size as usize, 
            align, 
            ptr::null_mut() as *mut u64
        )
    };
    assert!(!ptr.is_null(), "Failed to malloc");
    return Buf { len: size as usize, ptr };
}