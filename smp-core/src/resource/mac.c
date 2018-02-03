#include <stdio.h>
#include <sys/sysctl.h>
#include <sys/types.h>
#include <inttypes.h>

int memsize(int64_t* size)
{
  int mib[2] = { CTL_HW, HW_MEMSIZE };
  u_int namelen = sizeof(mib) / sizeof(mib[0]);
  size_t len = sizeof(size);
  return sysctl(mib, namelen, size, &len, NULL, 0);
}