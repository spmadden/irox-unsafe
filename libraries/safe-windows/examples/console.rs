use irox_safe_windows::error::Error;

fn main() -> Result<(), Error> {
    #[cfg(windows)]
    irox_safe_windows::term::dump_console_info()?;
    Ok(())
}
