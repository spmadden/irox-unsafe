use std::ffi::c_void;
use std::mem::size_of_val;

use windows::core::imp::CoTaskMemFree;
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    BOOL, ERROR_ACCESS_DENIED, ERROR_ACCOUNT_DISABLED, ERROR_CANCELLED, ERROR_INVALID_FLAGS,
    ERROR_INVALID_PARAMETER, ERROR_LOGON_FAILURE, ERROR_NOT_FOUND, ERROR_NO_SUCH_LOGON_SESSION,
    ERROR_PASSWORD_EXPIRED, HWND, WIN32_ERROR,
};
use windows::Win32::Graphics::Gdi::HBITMAP;
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredFree, CredPackAuthenticationBufferW, CredReadW,
    CredUIPromptForWindowsCredentialsW, CredUnPackAuthenticationBufferW, CredWriteW, CREDENTIALW,
    CREDUIWIN_CHECKBOX, CREDUIWIN_FLAGS, CREDUIWIN_GENERIC, CREDUI_INFOW,
    CRED_PACK_GENERIC_CREDENTIALS, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
};

use crate::error::Error;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum CredentialError {
    /// No Error!
    #[default]
    None,

    ///The user name or password is incorrect.
    ErrLogonFailure,

    /// The password for this account has expired.
    ErrPasswordExpired,

    /// This user can't sign in because this account is currently disabled.
    ErrAccountDisabled,

    /// Access is denied.
    ErrAccessDenied,

    /// Some other error code
    Other(u32),
}
impl From<CredentialError> for u32 {
    fn from(value: CredentialError) -> Self {
        match value {
            CredentialError::None => 0,
            CredentialError::ErrLogonFailure => ERROR_LOGON_FAILURE.0,
            CredentialError::ErrPasswordExpired => ERROR_PASSWORD_EXPIRED.0,
            CredentialError::ErrAccountDisabled => ERROR_ACCOUNT_DISABLED.0,
            CredentialError::ErrAccessDenied => ERROR_ACCESS_DENIED.0,
            CredentialError::Other(e) => e,
        }
    }
}

#[derive(Default)]
pub struct PromptOptions {
    subtitle_text: Option<HSTRING>,
    title_text: Option<HSTRING>,
    error: Option<CredentialError>,
}
impl PromptOptions {
    pub fn new() -> Self {
        PromptOptions::default()
    }
    #[must_use]
    pub fn with_title(self, title: &str) -> Self {
        PromptOptions {
            title_text: Some(HSTRING::from(title)),
            ..self
        }
    }
    #[must_use]
    pub fn with_subtitle(self, subtitle: &str) -> Self {
        PromptOptions {
            subtitle_text: Some(HSTRING::from(subtitle)),
            ..self
        }
    }
    #[must_use]
    pub fn with_error_code(self, err: CredentialError) -> Self {
        PromptOptions {
            error: Some(err),
            ..self
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub save_requested: bool,
}

fn error_code(code: u32) -> &'static str {
    match WIN32_ERROR(code) {
        ERROR_CANCELLED => "user cancelled",
        ERROR_INVALID_FLAGS => "invalid flags",
        ERROR_INVALID_PARAMETER => "invalid parameter",
        ERROR_NO_SUCH_LOGON_SESSION => "this credential manager can't be used",
        _ => "Unknown error",
    }
}

///
/// Displays the standard windows login prompt to ask for a username and password.
///
/// # Safety
/// Calls the windows functions [`CredUIPromptForWindowsCredentialsW`], [`CredUnPackAuthenticationBufferW`] and [`CoTaskMemFree`]
pub fn prompt(options: &PromptOptions) -> Result<Credentials, Error> {
    let psz_subtitle_text = options.subtitle_text.as_ref().map(HSTRING::as_ptr);
    let psz_title_text = options.title_text.as_ref().map(HSTRING::as_ptr);

    // can't chain the options above with these, as the ptr MUST outlive the slices
    let psz_title_text = psz_title_text.map(PCWSTR).unwrap_or(PCWSTR::null());
    let psz_subtitle_text = psz_subtitle_text.map(PCWSTR).unwrap_or(PCWSTR::null());

    let mut info = CREDUI_INFOW {
        cbSize: 0,
        hwndParent: HWND(0),
        pszMessageText: psz_subtitle_text,
        pszCaptionText: psz_title_text,
        hbmBanner: HBITMAP(0),
    };
    info.cbSize = size_of_val(&info) as u32;

    let mut username = [0u16; 255];
    let mut password = [0u16; 255];
    let mut should_save = BOOL(1);

    let puiinfo = Some(&info as *const CREDUI_INFOW);
    let dwautherror = options.error.unwrap_or_default().into();
    let save = Some(&mut should_save as *mut BOOL);
    let dwflags: CREDUIWIN_FLAGS = CREDUIWIN_CHECKBOX | CREDUIWIN_GENERIC;
    let mut pulauthpackage = [0u32];
    let mut puloutauthbuffersize = [0u32];

    let mut pcchlmaxusername = 255;
    let mut pcchlmaxpassword = 255;

    unsafe {
        let mut ppvoutauthbuffer: *mut c_void = std::ptr::null_mut();

        {
            let pulauthpackage = &mut pulauthpackage as *mut u32;
            let ppvoutauthbuffer = &mut ppvoutauthbuffer as *mut *mut c_void;
            let puloutauthbuffersize = &mut puloutauthbuffersize as *mut u32;
            let res = CredUIPromptForWindowsCredentialsW(
                puiinfo,              // Option<*const CREDUI_INFOA>
                dwautherror,          /* u32 */
                pulauthpackage,       // *mut u32
                None,                 // pvinauthbuffer: Option<*const c_void>
                0,                    // ulinauthbuffersize: u32
                ppvoutauthbuffer,     // *mut *mut c_void
                puloutauthbuffersize, // *mut u32
                save,                 // Option<*mut BOOL>
                dwflags,              // CREDUIWIN_FLAGS
            );
            if res != 0 {
                return Error::code(res, error_code(res));
            }
        }
        let dwflags = CRED_PACK_GENERIC_CREDENTIALS;

        let pszusername = PWSTR(username.as_mut_ptr());
        let pszpassword = PWSTR(password.as_mut_ptr());

        let res = CredUnPackAuthenticationBufferW(
            dwflags,                 // CRED_PACK_FLAGS,
            ppvoutauthbuffer,        // *const c_void,
            puloutauthbuffersize[0], // u32
            pszusername,             // PWSTR
            &mut pcchlmaxusername,   // *mut u32
            PWSTR::null(),           // PWSTR,
            None,                    // Option<*mut u32>
            pszpassword,             // PWSTR,
            &mut pcchlmaxpassword,   // *mut u32
        );

        CoTaskMemFree(ppvoutauthbuffer);

        if let Err(e) = res {
            return Err(e.into());
        }
    }

    let pcchlmaxusername = pcchlmaxusername.saturating_sub(1) as usize;
    let pcchlmaxpassword = pcchlmaxpassword.saturating_sub(1) as usize;
    let save_requested: bool = should_save.as_bool();
    let username = username
        .get(0..pcchlmaxusername)
        .map(String::from_utf16_lossy)
        .unwrap_or_default();
    let password = password
        .get(0..pcchlmaxpassword)
        .map(String::from_utf16_lossy)
        .unwrap_or_default();
    Ok(Credentials {
        username,
        password,
        save_requested,
    })
}

pub fn read_cred(target_name: &str) -> Result<Credentials, Error> {
    let target_name = str_2_widenullstr(target_name);
    let dtype = CRED_TYPE_GENERIC;
    unsafe {
        let mut cred_ptr = std::ptr::null_mut();
        let credential = &mut cred_ptr;
        let res = CredReadW(
            PCWSTR(target_name.as_ptr()), // LPCWSTR
            dtype,                        // DWORD
            0,                            // flags: DWORD
            credential,
        );
        if let Err(e) = res {
            CredFree(cred_ptr as *mut c_void);
            if e == ERROR_NOT_FOUND.into() {
                return Error::notfound();
            }
            return Err(e.into());
        }

        let cred = *cred_ptr;

        let blob_size = cred.CredentialBlobSize as usize;
        let blob: Vec<u8> = Vec::from(std::slice::from_raw_parts(cred.CredentialBlob, blob_size));
        let mut username = [0u16; 255];
        let mut password = [0u16; 255];
        let mut pcchlmaxusername = 255;
        let mut pcchlmaxpassword = 255;
        {
            let dwflags = CRED_PACK_GENERIC_CREDENTIALS;

            let pszusername = PWSTR(username.as_mut_ptr());
            let pszpassword = PWSTR(password.as_mut_ptr());

            CredUnPackAuthenticationBufferW(
                dwflags,                        // CRED_PACK_FLAGS,
                blob.as_ptr() as *const c_void, // *const c_void,
                blob_size as u32,               // u32
                pszusername,                    // PWSTR
                &mut pcchlmaxusername,          // *mut u32
                PWSTR::null(),                  // PWSTR,
                None,                           // Option<*mut u32>
                pszpassword,                    // PWSTR,
                &mut pcchlmaxpassword,          // *mut u32
            )?;
        }

        let pcchlmaxusername = pcchlmaxusername.saturating_sub(1) as usize;
        let pcchlmaxpassword = pcchlmaxpassword.saturating_sub(1) as usize;
        let username = username
            .get(0..pcchlmaxusername)
            .map(String::from_utf16_lossy)
            .unwrap_or_default();
        let password = password
            .get(0..pcchlmaxpassword)
            .map(String::from_utf16_lossy)
            .unwrap_or_default();

        CredFree(cred_ptr as *mut c_void);

        Ok(Credentials {
            username,
            password,
            save_requested: false,
        })
    }
}

pub fn write_cred(target_name: &str, comment: &str, creds: &Credentials) -> Result<(), Error> {
    let mut tgt_name = str_2_widenullstr(target_name);
    let mut comment = str_2_widenullstr(comment);

    let username = HSTRING::from(&creds.username);
    let mut usrname = str_2_widenullstr(&creds.username);
    let password = HSTRING::from(&creds.password);

    let mut packed_len = [255u32];
    let mut packed_buf = [0u8; 255];

    unsafe {
        let pszusername = PCWSTR(username.as_ptr());
        let pszpassword = PCWSTR(password.as_ptr());
        let len = &mut packed_len as *mut u32;
        let res = CredPackAuthenticationBufferW(
            CRED_PACK_GENERIC_CREDENTIALS, // dwflags
            pszusername,                   // LPWSTR
            pszpassword,                   // LPWSTR
            Some(packed_buf.as_mut_ptr()), // Option<*mut u8>
            len,
        );
        res?;
    }

    let cred = CREDENTIALW {
        Flags: Default::default(),
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(tgt_name.as_mut_ptr()),
        Comment: PWSTR(comment.as_mut_ptr()),
        LastWritten: Default::default(),
        CredentialBlobSize: packed_len[0],
        CredentialBlob: packed_buf.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        AttributeCount: 0,
        Attributes: std::ptr::null_mut(),
        TargetAlias: PWSTR::null(),
        UserName: PWSTR(usrname.as_mut_ptr()),
    };
    unsafe {
        CredWriteW(&cred as *const CREDENTIALW, 0)?;
    }
    Ok(())
}

pub fn read_or_prompt_and_save(
    target: &str,
    comment: &str,
    options: &PromptOptions,
) -> Result<Credentials, Error> {
    let err = match read_cred(target) {
        Ok(c) => {
            return Ok(c);
        }
        Err(e) => e,
    };
    if !err.is_notfound() {
        return Err(err);
    }

    let cred = prompt(options)?;

    if cred.save_requested {
        write_cred(target, comment, &cred)?;
    }

    Ok(cred)
}

pub fn delete_cred(target: &str) -> Result<(), Error> {
    let target = str_2_widenullstr(target);
    unsafe {
        if let Err(e) = CredDeleteW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, 0) {
            if e.code() != ERROR_NOT_FOUND.into() {
                return Err(e.into());
            }
        }
    }
    Ok(())
}

fn str_2_widenullstr(val: &str) -> Vec<u16> {
    let mut buf = HSTRING::from(val).as_wide().to_vec();
    buf.push(0);
    buf
}
