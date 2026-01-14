use std::ffi::{CStr, CString, c_char};

use libloading::{AsFilename, Library};

type DLLString = *const c_char;

type StringConsumer = unsafe extern "C" fn(DLLString);

type DLLResult<T> = Result<T, String>;

pub struct DLLib {
    library: Library,
}

pub unsafe fn load(filename: impl AsFilename) -> DLLResult<DLLib> {
    Ok(DLLib {
        library: unsafe { Library::new(filename).unwrap() },
    })
}

impl DLLib {
    unsafe fn create_string<F>(&self, lambda: F) -> Option<String>
    where
        F: FnOnce() -> *const c_char,
    {
        let raw_str = lambda();

        if raw_str.is_null() {
            None
        } else {
            let c_str = unsafe { CStr::from_ptr(raw_str) }
                .to_str()
                .unwrap()
                .to_owned();
            unsafe {
                self.library.get::<StringConsumer>(b"free_memory").unwrap()(raw_str);
            }
            Some(c_str)
        }
    }

    fn get_result(&self, maybestring: (DLLString, DLLString)) -> DLLResult<String> {
        if maybestring.1.is_null() {
            Ok(unsafe { self.create_string(|| maybestring.0).unwrap() })
        } else {
            Err(unsafe { self.create_string(|| maybestring.1).unwrap() })
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

        unsafe { (self.create_string(|| tup.0), self.create_string(|| tup.1)) }
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
}
