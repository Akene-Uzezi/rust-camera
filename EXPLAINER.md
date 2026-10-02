# Explainer for rustcamera

This document explains the Rust code in this project line by line, aimed at someone who knows Go but not Rust.

## 1. Rust concepts used in this file

### Imports
```rust
use std::ffi::CStr;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
```
These are the same as Go `import` statements. They bring names into scope so you can write `OpenOptions` instead of `std::fs::OpenOptions`.

### Structs and `repr(C)`
```rust
#[repr(C)]
struct V4l2Capability {
    driver: [u8; 16],
    card: [u8; 32],
    ...
}
```
A `struct` in Rust is similar to a Go struct. The difference here is `#[repr(C)]`. This tells Rust to lay out the fields in memory exactly the way C would, with no reordering and no extra padding tricks. We need this because the Linux kernel expects a C-compatible memory layout when we pass this struct to `ioctl`.

- `[u8; 16]` is a fixed-size array of 16 bytes. In Go you might write `[16]byte` or `[]byte`. Rust distinguishes between fixed-size arrays and slices (`[u8]` is a slice, which is like a Go slice).
- `u32` is an unsigned 32-bit integer.

### Derive macros: `#[derive(Default)]`
```rust
#[derive(Default)]
struct V4l2Capability { ... }
```
Rust does not automatically fill structs with zero values for you. The `derive` attribute tells the compiler to auto-generate an implementation of a trait (think interface). `Default` is a trait that provides a constructor called `V4l2Capability::default()`. Here it simply zeroes all bytes, which is exactly what we want for a C struct we are about to query.

In Go terms, this is roughly like having a function that returns `&V4l2Capability{}`.

### Unsafe C function binding
```rust
unsafe extern "C" {
    fn ioctl(fd: i32, request: u64, argp: *mut std::ffi::c_void) -> i32;
}
```
This declares the C function `ioctl` so Rust can call it. The `unsafe` keyword means the compiler cannot verify memory safety here, so the caller must promise to pass valid pointers and values.

- `extern "C"` uses the C ABI (calling convention).
- `*mut c_void` is a raw mutable pointer to an anonymous C type (`void *` in C). In Go this would be `unsafe.Pointer`.

### The ioctl request constant
```rust
const VIDIOC_QUERYCAP: u64 = (2u64 << 30)
    | (((std::mem::size_of::<V4l2Capability>() as u64) & 0x3fff) << 16)
    | (('V' as u64) << 8)
    | 0;
```
Linux ioctl uses a 32-bit request code composed from several fields. This constant encodes the `VIDIOC_QUERYCAP` ioctl for V4L2. It is pure bit manipulation:

- `2 << 30` is the ioctl type.
- The size of the struct goes into bits 16-29.
- `'V'` is the sequence character.
- `0` is the command number.

This is exactly the same macro expansion you would see in `<linux/videodev2.h>` in C.

### Main function and error returns
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
```
In Go, `main` cannot return an error. In Rust it can. Here `main` returns a `Result`:

- `Result<OkType, ErrType>` is an enum (tagged union) that is either `Ok(value)` or `Err(error)`.
- `()` is the unit type, like Go's empty struct, used when there is no meaningful success value.
- `Box<dyn std::error::Error>` is a heap-allocated trait object that can hold *any* error type. It is the Rust equivalent of Go's `error` interface.

### The `?` operator
```rust
let file = OpenOptions::new().read(true).write(true).open("/dev/video0")?;
```
The `?` operator unwraps a `Result`. If the value is `Ok(v)`, it yields `v`. If it is `Err(e)`, it immediately returns `Err(e)` from the current function. This is similar to writing:

```go
file, err := OpenOptions...open(...)
if err != nil {
    return nil, err
}
```

The big difference is that `?` does the early return automatically, and it works with any function that returns a `Result`.

### Opening the device
```rust
let file = OpenOptions::new()
    .read(true)
    .write(true)
    .open("/dev/video0")?;
```
`OpenOptions` is a builder that lets you configure exactly how a file is opened, similar to `os.OpenFile` with flags. Here we open `/dev/video0` for reading and writing.

### Raw file descriptor
```rust
let fd = file.as_raw_fd();
```
Unix systems represent open files as integers called file descriptors. `as_raw_fd()` extracts the raw integer. In Go this is essentially `file.Fd()`.

### Calling `ioctl`
```rust
let mut caps = V4l2Capability::default();
let result = unsafe {
    ioctl(
        fd,
        VIDIOC_QUERYCAP,
        &mut caps as *mut _ as *mut std::ffi::c_void,
    )
};
```
- `&mut caps` takes a mutable reference to `caps`. In Go terms, this is `&caps` but mutable.
- `as *mut _` casts the reference to a raw mutable pointer (`*mut V4l2Capability`).
- `as *mut c_void` casts it further to `*mut c_void`, matching the C signature of `ioctl`.
- Because this block dereferences raw pointers, it must be inside `unsafe { ... }`.

### Parsing C strings
```rust
let driver = CStr::from_bytes_until_nul(&caps.driver)?.to_str()?;
```
The kernel gave us byte arrays that are null-terminated C strings.

- `CStr::from_bytes_until_nul(&caps.driver)` reads bytes from the start of the array until it hits a null byte (`0`). It returns a `Result<CStr>` because the array might contain no null byte at all.
- `.to_str()?` converts the C string into a Rust `&str` (UTF-8 string slice), failing if the bytes are not valid UTF-8.

### Printing
```rust
println!("--- Camera Info ---");
println!("Driver: {}", driver);
```
These are Rust macros, not functions. `println!` writes to stdout; `eprintln!` writes to stderr. The `{}` inside the string is a placeholder filled by the arguments after the comma, like Go's `fmt.Printf`.

### Empty success return
```rust
Ok(())
```
Returning `Ok(())` from `main` means the program exited successfully. If any earlier `?` had failed, the function would have already returned `Err(...)`.

## 2. What the program does

This program queries a V4L2 (Video4Linux 2) video capture device on Linux and prints the driver name and card name.

## 3. How it does it

1. **Open `/dev/video0`** using `OpenOptions` with read and write permissions.
2. **Get the raw file descriptor** from the opened file.
3. **Prepare a `V4l2Capability` struct** zeroed with `Default::default()`.
4. **Call the C `ioctl` function** with the `VIDIOC_QUERYCAP` request code. This is the standard Linux mechanism for querying device capabilities.
5. **Check the return value**. If it is negative, the ioctl failed.
6. **Extract null-terminated strings** from the `driver` and `card` byte arrays inside the struct.
7. **Print the strings** to the terminal.

The core trick here is using raw FFI (`extern "C"`, `repr(C)`, and `unsafe`) to talk directly to the Linux kernel from safe Rust. Everything else is just plumbing around that system call.
