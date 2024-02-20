fn fail_with<E: std::fmt::Debug>(msg: &str, e: E) -> ! {
  eprintln!("[ERROR]: {} {:?}", msg, e);
  std::process::exit(1);
}

pub fn unwrap_or_fail<T, E: std::fmt::Debug>(
  result: Result<T, E>,
  msg: &str,
) -> T {
  match result {
    Ok(v) => v,
    Err(e) => fail_with(msg, e),
  }
}
