use std::{
    ffi::{CStr, CString, c_char},
    time::{Duration, SystemTime},
};

use libloading::{AsFilename, Library};

type DLLString = *const c_char;

type StringConsumer = unsafe extern "C" fn(DLLString);

type DLLResult<T> = Result<T, String>;

pub struct DLLib {
    library: Library,
}

#[derive(Debug)]
pub struct FileDetails {
    pub filename: String,
    pub date_modified: SystemTime,
}

pub unsafe fn load(filename: impl AsFilename) -> DLLResult<DLLib> {
    Ok(DLLib {
        library: unsafe { Library::new(filename).unwrap() },
    })
}

impl Drop for DLLib {
    fn drop(&mut self) {
        unsafe {
            self.library
                .get::<unsafe extern "C" fn() -> ()>(b"close_dll")
                .unwrap()();
        }
    }
}

impl DLLib {
    unsafe fn free_memory(&self, raw_str: DLLString) {
        unsafe { self.library.get::<StringConsumer>(b"free_memory").unwrap()(raw_str) }
    }

    unsafe fn create_string(&self, raw_str: DLLString) -> Option<String> {
        if raw_str.is_null() {
            None
        } else {
            let c_str = unsafe { CStr::from_ptr(raw_str) }
                .to_str()
                .unwrap()
                .to_owned();
            unsafe {
                self.free_memory(raw_str);
            }
            Some(c_str)
        }
    }

    fn get_result(&self, maybestring: (DLLString, DLLString)) -> DLLResult<String> {
        if maybestring.1.is_null() {
            Ok(unsafe { self.create_string(maybestring.0).unwrap() })
        } else {
            Err(unsafe { self.create_string(maybestring.1).unwrap() })
        }
    }

    pub unsafe fn validate(
        &self,
        credentials: &str,
        redirect_uri: &str,
    ) -> (Option<String>, Option<String>) {
        let credentials = CString::new(credentials).unwrap();
        let redirect_uri = CString::new(redirect_uri).unwrap();

        let tup = unsafe {
            self.library
                .get::<unsafe extern "C" fn(DLLString, DLLString) -> (DLLString, DLLString)>(
                    b"validate",
                )
                .unwrap()(credentials.as_ptr(), redirect_uri.as_ptr())
        };

        unsafe { (self.create_string(tup.0), self.create_string(tup.1)) }
    }

    pub unsafe fn extract_credentials(&self, uri: &str) -> DLLResult<String> {
        let cstring = CString::new(uri).unwrap();
        self.get_result(unsafe {
            self.library
                .get::<unsafe extern "C" fn(DLLString) -> (DLLString, DLLString)>(
                    b"extract_credentials",
                )
                .unwrap()(cstring.as_ptr())
        })
    }

    pub unsafe fn upload(
        &self,
        access_token: &str,
        filename: &str,
        date_modified: u64,
        data: Vec<u8>,
    ) -> DLLResult<()> {
        let access_token = CString::new(access_token).unwrap();
        let filename = CString::new(filename).unwrap();

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
                .unwrap()(
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
        let access_token = CString::new(access_token).unwrap();

        unsafe {
            let tup = self.library.get::<unsafe extern "C" fn(DLLString) -> (u64, *const (DLLString, u64), DLLString)>(b"read_cloud").unwrap()(access_token.as_ptr());

            if !tup.2.is_null() {
                Err(self.create_string(tup.2).unwrap())
            } else {
                let mut v: Vec<FileDetails> = Vec::new();

                for i in 0..tup.0 {
                    let detail = *tup.1.offset(i as isize);
                    v.push(FileDetails {
                        filename: self.create_string(detail.0 as DLLString).unwrap(),
                        date_modified: SystemTime::UNIX_EPOCH + Duration::from_secs(detail.1),
                    });
                }
                Ok(v)
            }
        }
    }

    pub unsafe fn download(&self, access_token: &str, filename: &str) {
        let access_token = CString::new(access_token).unwrap();
        let filename = CString::new(filename).unwrap();

        let tup = unsafe {
            self.library
                .get::<unsafe extern "C" fn(DLLString, DLLString) -> (DLLString, u64, DLLString)>(
                    b"download",
                )
                .unwrap()(access_token.as_ptr(), filename.as_ptr())
        };

        if let Some(err) = unsafe { self.create_string(tup.2) } {
            println!("Error in download: {err}")
        } else if tup.1 > 0 {
            let mut s = String::new();

            for i in 0..tup.1 {
                s.push(unsafe { *tup.0.offset(i as isize) } as u8 as char);
            }

            unsafe { self.free_memory(tup.0) };

            println!("{s}");
        }
    }
}
