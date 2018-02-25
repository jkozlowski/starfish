#include "libaio.h"
#include <xfs/xfs.h>
#include <xfs/linux.h>
#include <xfs/xfs_fs.h>
#include <sys/ioctl.h>

void io_prep_pread_c(struct iocb *iocb, int fd, void *buf, size_t count, long long offset) {
  io_prep_pread(iocb, fd, buf, count, offset);
}

void io_set_eventfd_c(struct iocb *iocb, int eventfd) {
  io_set_eventfd(iocb, eventfd);
}