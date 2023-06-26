use crate::{println, sbi};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "panic occurred in file '{}' at line {} : {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!(
            "panic occurred but can't get location information : {}",
            info.message().unwrap()
        );
    }
    sbi::shutdown()
}
