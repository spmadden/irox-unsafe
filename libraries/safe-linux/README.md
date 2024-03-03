IROX-SAFE-LINUX
================

*Pure-Rust implementations of the linux native API functions & [SYSCALLs] to make them ergonomic.  Effectively a drop-in replacement of glibc.*


Current Features:
------------------
* sys
  * [sysinfo](https://www.man7.org/linux/man-pages/man2/sysinfo.2.html)
* time
  * [clock_gettime](https://man7.org/linux/man-pages/man2/clock_gettime.2.html)
  * [clock_getres](https://man7.org/linux/man-pages/man2/clock_gettime.2.html)
  * [times](https://man7.org/linux/man-pages/man2/times.2.html)


[SYSCALLs]: https://man7.org/linux/man-pages/man2/syscalls.2.html