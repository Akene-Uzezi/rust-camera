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
pub struct V4l2PixFormat {
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

const fn ioctl_iowr<T>(type_: u8, nr: u8) -> u64 {
    (3u64 << 30)
        | (((std::mem::size_of::<T>() as u64) & 0x3fff) << 16)
        | ((type_ as u64) << 8)
        | (nr as u64)
}

const fn ioctl_iow<T>(type_: u8, nr: u8) -> u64 {
    (1u64 << 30)
        | (((std::mem::size_of::<T>() as u64) & 0x3fff) << 16)
        | ((type_ as u64) << 8)
        | (nr as u64)
}

const VIDIOC_S_FMT: u64 = ioctl_iowr::<V4l2Format>('V' as u8, 5);
const VIDIOC_REQBUFS: u64 = ioctl_iowr::<V4l2RequestBuffers>('V' as u8, 8);
const VIDIOC_QUERYBUF: u64 = ioctl_iowr::<V4l2Buffer>('V' as u8, 9);
const VIDIOC_QBUF: u64 = ioctl_iowr::<V4l2Buffer>('V' as u8, 15);
const VIDIOC_DQBUF: u64 = ioctl_iowr::<V4l2Buffer>('V' as u8, 17);
const VIDIOC_STREAMON: u64 = ioctl_iow::<u32>('V' as u8, 18);
const VIDIOC_STREAMOFF: u64 = ioctl_iow::<u32>('V' as u8, 19);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dev = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/video0")?;
    let fd = dev.as_raw_fd();
    let mut fmt = V4l2Format {
        type_: V4L2_BUF_TYPE_VIDEO_CAPTURE,
        fmt: [0; 200],
    };

    let pix = V4l2PixFormat {
        width: 640,
        height: 480,
        pixelformat: V4L2_PIX_FMT_MJPEG,
        field: 1, // V4L2_FIELD_ANY
        bytesperline: 0,
        sizeimage: 0,
        colorspace: 0,
        priv_val: 0,
        flags: 0,
        ycbcr_enc: 0,
        hsv_enc: 0,
        quantization: 0,
        xfer_func: 0,
    };

    unsafe {
        std::ptr::copy_nonoverlapping(
            &pix as *const _ as *const u8,
            fmt.fmt.as_mut_ptr(),
            std::mem::size_of::<V4l2PixFormat>(),
        );
        ioctl(fd, VIDIOC_S_FMT, &mut fmt as *mut _ as _);
    }

    let mut req = V4l2RequestBuffers {
        count: 1,
        type_: V4L2_BUF_TYPE_VIDEO_CAPTURE,
        memory: V4L2_MEMORY_MMAP,
        ..Default::default()
    };
    unsafe {
        ioctl(fd, VIDIOC_REQBUFS, &mut req as *mut _ as *mut _);
    }

    let mut buf = V4l2Buffer {
        type_: V4L2_BUF_TYPE_VIDEO_CAPTURE,
        memory: V4L2_MEMORY_MMAP,
        index: 0,
        ..Default::default()
    };
    unsafe {
        ioctl(fd, VIDIOC_QUERYBUF, &mut buf as *mut _ as *mut _);
    }

    let buffer_ptr = unsafe {
        mmap(
            std::ptr::null_mut(),
            buf.length as usize,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd,
            buf.m_offset as i64,
        )
    };

    if buffer_ptr == MAP_FAILED {
        return Err("mmap failed".into());
    }

    unsafe {
        ioctl(fd, VIDIOC_QBUF, &mut buf as *mut _ as *mut _);
        let mut type_ = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        ioctl(fd, VIDIOC_STREAMON, &mut type_ as *mut _ as *mut _);
    }

    println!("Capturing frame");
}
