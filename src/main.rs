use std::ffi::CStr;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

#[repr(C)]
#[derive(Default)]
struct V4l2Capability {
    driver: [u8; 16],
    card: [u8; 32],
    bus_info: [u8; 32],
    version: u32,
    capabilities: u32,
    device_caps: u32,
    reserved: [u32; 3],
}

unsafe extern "C" {
    fn ioctl(fd: i32, request: u64, argp: *mut std::ffi::c_void) -> i32;
}

const VIDIOC_QUERYCAP: u64 = (2u64 << 30)
    | (((std::mem::size_of::<V4l2Capability>() as u64) & 0x3fff) << 16)
    | (('V' as u64) << 8)
    | 0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/video0")?;

    let fd = file.as_raw_fd();
    let mut caps = V4l2Capability::default();
    let result = unsafe {
        ioctl(
            fd,
            VIDIOC_QUERYCAP,
            &mut caps as *mut _ as *mut std::ffi::c_void,
        )
    };

    if result < 0 {
        eprintln!("Failed to query video device");
        return Ok(());
    }

    let driver = CStr::from_bytes_until_nul(&caps.driver)?.to_str()?;
    let card = CStr::from_bytes_until_nul(&caps.card)?.to_str()?;

    println!("--- Camera Info ---");
    println!("Driver: {}", driver);
    println!("Card: {}", card);

    Ok(())
}
