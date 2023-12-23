use irox_safe_windows::credentials::{
    delete_cred, read_cred, read_or_prompt_and_save, PromptOptions,
};
use irox_safe_windows::error::Error;
#[allow(clippy::print_stdout)]
#[cfg(windows)]
pub fn main() -> Result<(), Error> {
    let target = "irox-safe-windows-test-cred";
    let options = PromptOptions::new()
        .with_title("Little Title Text!")
        .with_subtitle("Big Title Text!");
    let cred = read_or_prompt_and_save(target, "test comment", &options)?;
    println!("{cred:?}");

    if cred.save_requested {
        let cred = read_cred(target)?;
        println!("{cred:?}");
    }

    delete_cred(target)?;

    Ok(())
}

#[cfg(not(windows))]
pub fn main() {
    eprintln!("This example only supported on windows targets.");
}
