pub(crate) fn log(message: &str, func_name: Option<&str>) -> () {
    if let Some(fname) = func_name {
        println!("{} [{}]: {}", chrono::Local::now(), fname, message);
    }
    println!("{}: {}",chrono::Local::now(),  message);
}
