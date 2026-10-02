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

#[repr(C)]
pub struct V4L2PixFormat {
    pub width: u32,
    pub height: u32,
    pub pixelformat: u32,
    pub field: u32,
    pub bytesperline: u32,
    pub sizeimage: u32,
    pub colorspace: u32,
    pub priv_val: u32,
    pub flags: u32,
    pub ycbcr_enc: u32,
    pub hsv_enc: u32,
    pub quantization: u32,
    pub xfer_func: u32,
}

#[repr(C)]
pub struct V4l2Format {
    pub type_: u32,
    pub fmt: [u8; 200], // Raw union buffer padding
}

#[repr(C)]
#[derive(Default)]
pub struct V4l2RequestBuffers {
    pub count: u32,
    pub type_: u32,
    pub memory: u32,
    pub reserved: [u32; 2],
}

#[repr(C)]
#[derive(Default)]
pub struct V4l2Buffer {
    pub index: u32,
    pub type_: u32,
    pub bytesused: u32,
    pub flags: u32,
    pub field: u32,
    pub timestamp_sec: i64,
    pub timestamp_usec: i64,
    pub timecode: [u8; 12],
    pub sequence: u32,
    pub memory: u32,
    pub m_offset: u32,
    pub length: u32,
    pub reserved2: u32,
    pub request_fd: i32,
}

fn main() {}
