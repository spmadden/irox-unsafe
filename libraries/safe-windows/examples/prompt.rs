use irox_safe_windows::credentials::{prompt, Credentials, PromptOptions};
use irox_safe_windows::error::Error;

#[allow(clippy::print_stdout)]
#[cfg(windows)]
pub fn main() -> Result<(), Error> {
    let options = PromptOptions::new()
        .with_title("Little Title Text!")
        .with_subtitle("Big Title Text!");
    let creds: Credentials = prompt(&options)?;
    let username: &String = &creds.username;
    let password: &String = &creds.password;
    let user_requested_save: &bool = &creds.save_requested;

    println!("User: {username}");
    println!("Password: {password}");
    println!("Save Checkbox Selected: {user_requested_save}");

    Ok(())
}

#[cfg(not(windows))]
pub fn main() {
    eprintln!("This example only supported on windows targets.");
}
