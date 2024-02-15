pub fn fail_with(msg: &str, e: crate::io::Error) -> ! {
    eprintln!("[ERROR]: {} {:?}", msg, e);
    std::process::exit(1);
}
