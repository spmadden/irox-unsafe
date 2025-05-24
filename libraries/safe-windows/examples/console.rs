use irox_safe_windows::error::Error;
use irox_safe_windows::term::dump_console_info;

fn main() -> Result<(), Error> {
    dump_console_info()?;
    Ok(())
}
