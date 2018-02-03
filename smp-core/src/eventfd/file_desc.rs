use std::os::unix::io::{RawFd, AsRawFd, FromRawFd};
use libc;
use libc::size_t;
use libc::c_void;
use std::io;

// Copied from mio

#[derive(Debug)]
pub struct Io {
    fd: RawFd,
}

impl Io {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe {
            libc::read(self.fd,
                       buf.as_mut_ptr() as *mut c_void,
                       buf.len() as size_t)
        };
        Ok(ret as usize)
    }
}

impl AsRawFd for Io {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl FromRawFd for Io {
    unsafe fn from_raw_fd(fd: RawFd) -> Io {
        Io { fd: fd }
    }
}

impl Drop for Io {
    fn drop(&mut self) {
        use nix::unistd::close;
        let _ = close(self.as_raw_fd());
    }
}
