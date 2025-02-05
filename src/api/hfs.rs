use crate::api::process::ExitCode;

pub fn check_hfs_bounds(arg: &str) -> Result<(), ExitCode> {
    if arg == "/hfs" || arg == "/hfs/" || arg.starts_with("/hfs/") {
        #[cfg(not(test))]
        error!("Permission denied. HFS is non-accessible by users.");
        return Err(ExitCode::Failure);
    }
    if crate::sys::process::dir() == "/" && (arg == "hfs" || arg == "hfs/" || arg.starts_with("hfs/")) {
        #[cfg(not(test))]
        error!("Permission denied. HFS is non-accessible by users.");
        return Err(ExitCode::Failure);
    }
    Ok(())
}