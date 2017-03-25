use eventfd::EventfdFd;
use eventfd::file_desc::Io;
use futures::Async;
use futures::Future;
use futures::Poll;
use futures::unsync::oneshot::channel;
use futures::unsync::oneshot::Sender;
use futures::unsync::oneshot::Receiver;
use libc::c_int;
use libc::c_long;
use libc::c_void;
use libc::size_t;
use nix;
use nix::NixPath;
use nix::Errno;
use nix::fcntl;
use nix::fcntl::OFlag;
use nix::sys::stat;
use nix::sys::statfs;
use nix::sys::statfs::vfs::Statfs;
use nix::sys::statfs::vfs::TMPFS_MAGIC;
use slog::Logger;
use std;
use std::boxed::Box;
use std::cell::Cell;
use std::cell::RefCell;
use std::convert::AsMut;
use std::io;
use std::io::Read;
use std::mem;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::RawFd;
use std::ptr;
use std::rc::Rc;
use std::vec::Vec;
use tokio_core::reactor::Handle;
use tokio_core::reactor::PollEvented;

// TODO:
// # Open file should delegate to another thread.
// # Periodic flushing, instead of flushing on every call to #submit_io.
// # Handle errors in #flush_pending_aio.
// # Query dma alignment
// # Switch to #feature("raw") when storing the Completion in iocb.
// # Flush out File implementation
// # Move everything behind traits, so there can be threadpool backed
//   version for Windows (or completion ports?).

include!(concat!(env!("OUT_DIR"), "/aio.rs"));

pub struct IoContext {
    inner: aio::io_context_t,
}

impl Drop for IoContext {
    fn drop(&mut self) {
        unsafe {
            aio::io_destroy(self.inner);
        }
    }
}

#[allow(dead_code)]
pub struct FileManager {
    log: Logger,
    io: Rc<RefCell<PollEvented<EventfdFd>>>,

    io_context: Rc<IoContext>,
    pending_aio: RefCell<Vec<aio::iocb>>,

    // Stats
    aio_reads: Cell<u64>,
    aio_read_bytes: Cell<u64>,
    aio_writes: Cell<u64>,
    aio_write_bytes: Cell<u64>,
}

const MAX_IO: usize = 128;

impl FileManager {
    pub fn create(log: Logger, handle: &Handle) -> io::Result<FileManager> {
        trace!(log, "FileManager::create");

        let eventfd = try!(EventfdFd::new());
        let io = try!(PollEvented::new(eventfd, handle));

        let file_manager = FileManager::new(log, io);
        file_manager.setup_iopoll(handle);
        Ok(file_manager)
    }

    pub fn open_file_dma<P: ?Sized + NixPath>(&self, path: &P, flags: OFlag) -> io::Result<File> {
        //  return _thread_pool.submit<syscall_result<int>>([name, flags, options, strict_o_direct = _strict_o_direct] {
        //  static constexpr mode_t mode = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH; // 0644
        let mode = stat::S_IRUSR | stat::S_IWUSR | stat::S_IRGRP | stat::S_IROTH; // 0644

        //  // We want O_DIRECT, except in two cases:
        //  //   - tmpfs (which doesn't support it, but works fine anyway)
        //  //   - strict_o_direct == false (where we forgive it being not supported)
        //  // Because open() with O_DIRECT will fail, we open it without O_DIRECT, try
        //  // to update it to O_DIRECT with fcntl(), and if that fails, see if we
        //  // can forgive it.
        //  auto is_tmpfs = [] (int fd) {
        let check_is_tmpfs = |fd: RawFd| -> nix::Result<bool> {
            struct MyRawFd {
                inner: RawFd,
            }
            impl AsRawFd for MyRawFd {
                fn as_raw_fd(&self) -> RawFd {
                    self.inner
                }
            }
            let rawfd = MyRawFd { inner: fd };
            //  struct ::statfs buf;
            let mut buf: Statfs = unsafe { mem::uninitialized() };
            //  auto r = ::fstatfs(fd, &buf);
            let _ = try!(statfs::fstatfs(&rawfd, &mut buf));
            //  return buf.f_type == 0x01021994; // TMPFS_MAGIC
            Ok(buf.f_type == TMPFS_MAGIC)
        };

        //  auto open_flags = O_CLOEXEC | static_cast<int>(flags);
        let open_flags = fcntl::O_CLOEXEC | flags;

        //  int fd = ::open(name.c_str(), open_flags, mode);
        let fd = try!(fcntl::open(path, open_flags, mode));
        let io = unsafe { Io::from_raw_fd(fd) };

        //  int r = ::fcntl(fd, F_SETFL, open_flags | O_DIRECT);
        let _ = fcntl::fcntl(fd, fcntl::FcntlArg::F_SETFL(open_flags | fcntl::O_DIRECT));

        //  auto maybe_ret = wrap_syscall<int>(r);  // capture errno (should be EINVAL)
        let _ = try!(check_is_tmpfs(fd));
        //  if (r == -1  && strict_o_direct && !is_tmpfs(fd)) {
        //      ::close(fd);
        //      return maybe_ret;
        //  }
        //  if (fd != -1) {
        //      fsxattr attr = {};
        //      if (options.extent_allocation_size_hint) {
        //          attr.fsx_xflags |= XFS_XFLAG_EXTSIZE;
        //          attr.fsx_extsize = options.extent_allocation_size_hint;
        //      }
        //      // Ignore error; may be !xfs, and just a hint anyway
        //      ::ioctl(fd, XFS_IOC_FSSETXATTR, &attr);
        //  }
        //  return wrap_syscall<int>(fd);
        //  }).then([options] (syscall_result<int> sr) {
        //      sr.throw_if_error();
        //      return make_ready_future<file>(file(sr.result, options));
        //  });
        Ok(File::new(io, self))
    }

    fn new(log: Logger, io: PollEvented<EventfdFd>) -> FileManager {
        let mut io_context = IoContext { inner: ptr::null_mut() };
        FileManager::io_setup(&mut io_context.inner);

        let file_manager = FileManager {
            log: log,
            io: Rc::new(RefCell::new(io)),
            io_context: Rc::new(io_context),
            pending_aio: RefCell::new(Vec::new()),
            aio_reads: Cell::new(0),
            aio_read_bytes: Cell::new(0),
            aio_writes: Cell::new(0),
            aio_write_bytes: Cell::new(0),
        };

        file_manager
    }

    fn io_setup(io_context: *mut aio::io_context_t) -> nix::Result<()> {
        let res = unsafe { aio::io_setup(MAX_IO as c_int, io_context) };
        Errno::result(res).map(|_| ())
    }

    fn setup_iopoll(&self, handle: &Handle) {
        let io_poll = IoPoll {
            log: self.log.clone(),
            io_context: self.io_context.clone(),
            io: self.io.clone(),
        };
        let io_poll_unit = io_poll.map_err(|_| ());
        handle.spawn(io_poll_unit)
    }

    fn submit_io_read<F, R>(&self, f: F) -> Receiver<R>
        where F: FnOnce(*mut aio::iocb, Sender<R>) -> Box<Completion>
    {
        self.aio_reads.set(self.aio_reads.get() + 1);
        trace!(self.log, "submit_io_read"; "aio_reads" => self.aio_reads.get());
        self.submit_io(f)
    }

    fn submit_io<F, R>(&self, prepare_io: F) -> Receiver<R>
        where F: FnOnce(*mut aio::iocb, Sender<R>) -> Box<Completion>
    {
        let (sender, receiver) = channel();

        let mut io = aio::iocb::default();
        let completion = Box::new(prepare_io(&mut io as *mut aio::iocb, sender));

        // if (_aio_eventfd) {
        // io_set_eventfd(&io, _aio_eventfd->get_fd());
        unsafe {
            io_set_eventfd_c(&mut io as *mut aio::iocb,
                             (*(*self.io).borrow()).get_ref().as_raw_fd());
        }
        // }

        io.data = Box::into_raw(completion) as *mut Box<Completion> as *mut std::os::raw::c_void;

        //  _pending_aio.push_back(io);
        let _ = self.push_iocb(io);

        //  pr.release();

        //  if ((_io_queue->queued_requests() > 0) ||
        //    (_pending_aio.size() >= std::min(max_aio / 4, _io_queue->_capacity / 2))) {
        //        if pending_aio_len >= MAX_IO {
        //  flush_pending_aio();
        self.flush_pending_aio();
        //        }

        receiver
    }

    fn push_iocb(&self, iocb: aio::iocb) -> usize {
        let mut pending_aio = self.pending_aio.borrow_mut();
        pending_aio.push(iocb);
        pending_aio.len()
    }

    //    fn throw_kernel_error(value: c_ulong) -> io::Result<usize> {
    //        if value < 0 {
    //            Err(io::Error::from_raw_os_error(value as i32))
    //        } else {
    //            Ok(value as usize)
    //        }
    //    }

    fn flush_pending_aio(&self) -> bool {
        let mut did_work = false;

        let mut pending_aio = self.pending_aio.borrow_mut();

        while !(*pending_aio).is_empty() {
            let nr: usize = pending_aio.len();

            let r = {
                let mut iocbs: Vec<&aio::iocb> = Vec::with_capacity(nr);
                for elem in &mut *pending_aio {
                    iocbs.push(elem);
                }

                unsafe {
                    let ptr = iocbs.as_mut_ptr() as *mut *mut aio::iocb;
                    aio::io_submit(self.io_context.inner, nr as i64, ptr)
                }
            };

            let mut nr_consumed: usize = 0;
            if r < 0 {
                trace!(self.log, "Result"; "r" => r);
                // TODO: handle the problems
                //      auto ec = -r;
                //      switch (ec) {
                //          case EAGAIN:
                //              return did_work;
                //          case EBADF: {
                //              auto pr = reinterpret_cast<promise<io_event>*>(iocbs[0]->data);
                //              try {
                //                  throw_kernel_error(r);
                //              } catch (...) {
                //                  pr->set_exception(std::current_exception());
                //              }
                //              delete pr;
                //              _io_context_available.signal(1);
                //              nr_consumed = 1;
                //              break;
                //          }
                //          default:
                //              throw_kernel_error(r);
                //              abort();
                //      }
            } else {
                nr_consumed = r as usize;
                trace!(self.log, "nr consumed"; "nr_consumed" => nr_consumed);
            }

            did_work = true;

            if nr_consumed == nr {
                pending_aio.clear();
            } else {
                pending_aio.drain(0..nr);
            }
        }

        did_work
    }
}

/// The way this works is that we hide the type of the buffer behind
/// the trait. The implementation then owns the buffer T,
/// and the pointer to underlying data are stored in the iocb.
trait Completion {
    fn complete(self: Box<Self>, result: usize);
}

struct CompletionImpl<T> {
    buf: T,
    sender: Sender<(T, usize)>,
}

impl<T> Completion for CompletionImpl<T>
    where T: AsMut<[u8]> + Clone
{
    fn complete(self: Box<Self>, result: usize) {
        let buf = self.buf.clone();
        self.sender.send((buf, result));
    }
}

/// A data file on persistent storage.
///
/// File objects represent uncached, unbuffered files.  As such great care
/// must be taken to cache data at the application layer; neither tokio-file-aio
/// nor the OS will cache these file.
///
/// Data is transferred using direct memory access (DMA).  This imposes
/// restrictions on file offsets and data pointers.  The former must be aligned
/// on a 4096 byte boundary, while a 512 byte boundary suffices for the latter.
#[allow(dead_code)]
pub struct File<'a> {
    inner: Io,
    manager: &'a FileManager,
    memory_dma_alignment: u64,
    disk_read_dma_alignment: u64,
    disk_write_dma_alignment: u64,
}

impl<'a> File<'a> {
    pub fn new(io: Io, manager: &'a FileManager) -> File<'a> {
        let mut file = File {
            inner: io,
            manager: manager,
            memory_dma_alignment: 4096,
            disk_read_dma_alignment: 4096,
            disk_write_dma_alignment: 4096,
        };

        File::query_dma_alignment(&mut file);

        file
    }

    fn query_dma_alignment(_file: &mut File) {
        //dioattr da;
        //auto r = ioctl(_fd, XFS_IOC_DIOINFO, &da);
        //if (r == 0) {
        //_memory_dma_alignment = da.d_mem;
        //_disk_read_dma_alignment = da.d_miniosz;
        //// xfs wants at least the block size for writes
        //// FIXME: really read the block size
        //_disk_write_dma_alignment = std::max<unsigned>(da.d_miniosz, 4096);
        //}
        //}
    }

    pub fn read_dma<T>(&self, aligned_pos: usize, mut aligned_buf: T) -> Receiver<(T, usize)>
        where T: AsMut<[u8]> + Clone + 'static
    {
        self.manager.submit_io_read(move |iocb, sender| unsafe {
            io_prep_pread_c(iocb,
                            self.inner.as_raw_fd(),
                            aligned_buf.as_mut().as_mut_ptr() as *mut c_void,
                            aligned_buf.as_mut().len() as size_t,
                            aligned_pos as i64);
            Box::new(CompletionImpl {
                         buf: aligned_buf,
                         sender: sender,
                     })
        })
    }
}

extern "C" {
    pub fn io_prep_pread_c(iocb: *mut aio::iocb,
                           fd: c_int,
                           buf: *mut c_void,
                           count: size_t,
                           offset: c_long);
    pub fn io_set_eventfd_c(iocb: *mut aio::iocb, eventfd: c_int);
}

struct IoPoll {
    log: Logger,
    io: Rc<RefCell<PollEvented<EventfdFd>>>,
    io_context: Rc<IoContext>,
}

impl Future for IoPoll {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut io = (*self.io).borrow_mut();
        if let Async::NotReady = io.poll_read() {
            return Ok(Async::NotReady);
        }
        let mut buf: [u8; 8] = [0; 8];
        match io.read(&mut buf) {
            Ok(_) => {
                self.process_io();
                io.need_read();
                Ok(Async::NotReady)
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                io.need_read();
                Ok(Async::NotReady)
            }
            Err(e) => Err(e),
        }
    }
}

impl IoPoll {
    fn process_io(&self) -> usize {
        trace!(self.log, "process_io: START");

        let mut ev: [aio::io_event; MAX_IO] = unsafe { mem::uninitialized() };

        let mut timeout = aio::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        let n = unsafe {
            aio::io_getevents(self.io_context.inner,
                              1,
                              MAX_IO as i64,
                              ev.as_mut_ptr() as *mut aio::io_event,
                              &mut timeout as *mut aio::timespec) as usize
        };

        assert!(n < MAX_IO);

        trace!(self.log, "process_io: {:?} events", n);

        for e in &mut ev[0..n] {
            let pr: Box<Box<Completion>> = unsafe { Box::from_raw(e.data as *mut Box<Completion>) };

            e.data = ptr::null_mut();

            let r = e.res as usize;

            pr.complete(r)
        }

        trace!(self.log, "process_io: END");

        n
    }
}
