pub mod file_desc;

use eventfd::file_desc::Io;
use libc;
use mio::Evented;
use mio::Poll;
use mio::PollOpt;
use mio::Ready;
use mio::Token;
use nix;
use nix::Errno;
use std::io;
use mio::unix::EventedFd;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::RawFd;
use std::io::Read;

pub struct EventfdFd {
    eventfd_io: Io,
}

impl EventfdFd {
    pub fn new() -> io::Result<EventfdFd> {
        EventfdFd::to_io(0, 0)
    }

    pub fn parent_to_child() -> io::Result<EventfdFd> {
        EventfdFd::to_io(0, libc::EFD_CLOEXEC)
    }

    fn to_io(init: libc::c_uint, flags: libc::c_int) -> io::Result<EventfdFd> {
        let fd = try!(EventfdFd::eventfd(init, flags));
        Ok(From::from(unsafe { Io::from_raw_fd(fd) }))
    }

    fn eventfd(init: libc::c_uint, flags: libc::c_int) -> nix::Result<RawFd> {
        let res = unsafe { libc::eventfd(init, flags) };
        Errno::result(res).map(|r| r as RawFd)
    }
}

impl Evented for EventfdFd {
    fn register(&self,
                poll: &Poll,
                token: Token,
                interest: Ready,
                opts: PollOpt)
                -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(&self,
                  poll: &Poll,
                  token: Token,
                  interest: Ready,
                  opts: PollOpt)
                  -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}

impl AsRawFd for EventfdFd {
    fn as_raw_fd(&self) -> RawFd {
        self.eventfd_io.as_raw_fd()
    }
}

impl From<Io> for EventfdFd {
    fn from(io: Io) -> EventfdFd {
        EventfdFd { eventfd_io: io }
    }
}

impl Read for EventfdFd {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.eventfd_io.read(buf)
    }
}
