#[panic_handler]
fn panic_handler (info: &core::panic::PanicInfo) -> ! {
    let err = info.message().unwrap();
    println!("{}", err);

    if let Some(location) = info.location() {
        println!("Panicked at file: {}, line: {}, message: {}", location.file(), location.line(), err );
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }

    // 不知道返回什么错误码，直接loop即可
    loop{}
}



