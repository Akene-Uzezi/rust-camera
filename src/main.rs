use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::io::AsRawFd;

unsafe extern "C" {
    fn ioctl(fd: i32, request: u64, argp: *mut std::ffi::c_void) -> i32;
    fn mmap(
        addr: *mut std::ffi::c_void,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut std::ffi::c_void;
    fn munmap(addr: *mut std::ffi::c_void, length: usize) -> i32;
}

const PROT_READ: i32 = 0x1;
const PROT_WRITE: i32 = 0x2;
const MAP_SHARED: i32 = 0x01;
const MAP_FAILED: *mut std::ffi::c_void = !0 as *mut std::ffi::c_void;

const V4L2_BUF_TYPE_VIDEO_CAPTURE: u32 = 1;
const V4L2_MEMORY_MMAP: u32 = 1;

const V4L2_PIX_FMT_MJPEG: u32 = u32::from_le_bytes(*b"MJPG");

fn main() {}
