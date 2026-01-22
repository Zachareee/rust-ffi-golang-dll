use std::{
    ffi::{CStr, CString, c_char},
    time::{Duration, SystemTime},
};

use libloading::{AsFilename, Library};

type DLLString = *const c_char;
type DLLFileDetails = *const (DLLString, u64);
type DLLResult<T> = Result<T, String>;

pub struct DLLib {
    library: Library,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Info {
    pub name: String,
    pub description: String,
    pub author: String,
    pub icon_url: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FileDetails {
    pub filename: String,
    pub date_modified: SystemTime,
}

pub unsafe fn load(filename: impl AsFilename) -> DLLResult<DLLib> {
    Ok(DLLib {
        library: unsafe { Library::new(filename).unwrap() },
    })
}

impl DLLib {
    unsafe fn free_string(&self, raw_str: DLLString) {
        unsafe {
            self.library
                .get::<unsafe extern "C" fn(DLLString)>(b"free_string")
                .expect("free_string function not found")(raw_str)
        }
    }

    unsafe fn create_string(&self, raw_str: DLLString) -> Option<String> {
        if raw_str.is_null() {
            None
        } else {
            let c_str = unsafe { CStr::from_ptr(raw_str) }
                .to_str()
                .unwrap_or_default()
                .to_owned();
            unsafe {
                self.free_string(raw_str);
            }
            Some(c_str)
        }
    }

    pub unsafe fn info(&self) -> Info {
        let (name, description, author, icon_url) = unsafe {
            self.library
                .get::<unsafe extern "C" fn() -> (DLLString, DLLString, DLLString, DLLString)>(
                    b"info",
                )
                .expect("info function not found")()
        };

        unsafe {
            Info {
                name: self.create_string(name).unwrap(),
                description: self.create_string(description).unwrap(),
                author: self.create_string(author).unwrap(),
                icon_url: self.create_string(icon_url).unwrap(),
            }
        }
    }

    pub unsafe fn validate(
        &self,
        credentials: &str,
        redirect_uri: &str,
    ) -> (Option<String>, Option<String>) {
        let credentials = CString::new(credentials).unwrap_or_default();
        let redirect_uri = CString::new(redirect_uri).unwrap_or_default();

        let (url, msg) = unsafe {
            self.library
                .get::<unsafe extern "C" fn(DLLString, DLLString) -> (DLLString, DLLString)>(
                    b"validate",
                )
                .expect("validate function not found")(
                credentials.as_ptr(), redirect_uri.as_ptr()
            )
        };

        unsafe { (self.create_string(url), self.create_string(msg)) }
    }

    pub unsafe fn extract_credentials(&self, uri: &str) -> DLLResult<String> {
        let cstring = CString::new(uri).unwrap_or_default();

        let (res, possible_err) = unsafe {
            self.library
                .get::<unsafe extern "C" fn(DLLString) -> (DLLString, DLLString)>(
                    b"extract_credentials",
                )
                .expect("extract_credentials function not found")(cstring.as_ptr())
        };

        if let Some(err) = unsafe { self.create_string(possible_err) } {
            Err(err)
        } else {
            Ok(unsafe {
                self.create_string(res)
                    .expect("Both ok and error value are empty")
            })
        }
    }

    pub unsafe fn upload(
        &self,
        access_token: &str,
        filename: &str,
        date_modified: u64,
        data: Vec<u8>,
    ) -> DLLResult<()> {
        let access_token = CString::new(access_token).unwrap_or_default();
        let filename = CString::new(filename).unwrap_or_default();

        unsafe {
            let ptr = self
                .library
                .get::<unsafe extern "C" fn(
                    DLLString,
                    DLLString,
                    u64,
                    DLLString,
                    u64,
                ) -> DLLString>(b"upload")
                .expect("upload function not found")(
                access_token.as_ptr(),
                filename.as_ptr(),
                date_modified,
                data.as_ptr() as *const i8,
                data.len() as u64,
            );
            self.create_string(ptr).map_or(Ok(()), |e| Err(e))
        }
    }

    pub unsafe fn read_cloud(&self, access_token: &str) -> DLLResult<Vec<FileDetails>> {
        let access_token = CString::new(access_token).unwrap_or_default();

        unsafe {
            let (ptr, count, possible_err) =
                self.library
                    .get::<unsafe extern "C" fn(DLLString) -> (DLLFileDetails, u64, DLLString)>(
                        b"read_cloud",
                    )
                    .expect("read_cloud function not found")(access_token.as_ptr());

            if let Some(err) = self.create_string(possible_err) {
                Err(err)
            } else {
                let mut v: Vec<FileDetails> = Vec::new();

                for i in 0..count as isize {
                    let detail = *ptr.offset(i);
                    v.push(FileDetails {
                        filename: self.create_string(detail.0 as DLLString).unwrap(),
                        date_modified: SystemTime::UNIX_EPOCH + Duration::from_secs(detail.1),
                    });
                }

                self.library
                    .get::<unsafe extern "C" fn(u64, DLLFileDetails)>(b"free_file_details")
                    .expect("free_file_details function not found")(count, ptr);
                Ok(v)
            }
        }
    }

    pub unsafe fn download(&self, access_token: &str, filename: &str) -> DLLResult<Vec<u8>> {
        let access_token = CString::new(access_token).unwrap_or_default();
        let filename = CString::new(filename).unwrap_or_default();

        let (ptr, count, possible_err) =
            unsafe {
                self.library
                .get::<unsafe extern "C" fn(DLLString, DLLString) -> (DLLString, u64, DLLString)>(
                    b"download",
                )
                .expect("download function not found")(access_token.as_ptr(), filename.as_ptr())
            };

        if let Some(err) = unsafe { self.create_string(possible_err) } {
            Err(err)
        } else {
            let mut v = Vec::new();
            let u8_ptr = ptr as *const u8;

            for i in 0..count as isize {
                v.push(unsafe { *u8_ptr.offset(i) });
            }

            unsafe { self.free_string(ptr) };

            Ok(v)
        }
    }
}
