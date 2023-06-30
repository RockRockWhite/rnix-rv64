use crate::print;

const FD_STDOUT: usize = 1;
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str_to_write = core::str::from_utf8(slice).unwrap();
            print!("{}", str_to_write);
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
