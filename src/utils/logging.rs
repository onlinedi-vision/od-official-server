/// Function used for... logging.
///
/// # Examples
/// ```rs
/// #[named]
/// pub fn my_func() -> () {
///     logging::log("SERVERS FAIL: fetch_server_channels", Some(function_name!()));
/// }
/// ```
pub(crate) fn log(message: &(impl std::fmt::Debug + std::fmt::Display + ?Sized), func_name: Option<&str>) {
    if let Some(fname) = func_name {
        println!("{} [{}]: {}", chrono::Local::now(), fname, message);
    }
    println!("{}: {}",chrono::Local::now(),  message);
}
