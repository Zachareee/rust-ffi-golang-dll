use std::ffi::{CStr, c_char};

use libloading::{AsFilename, Library, Symbol};

type StringSupplier = unsafe extern "C" fn() -> *const c_char;
type StringConsumer = unsafe extern "C" fn(*const c_char);

pub struct DLLib {
    library: Library,
}

pub fn load(filename: impl AsFilename) -> DLLib {
    unsafe {
        DLLib {
            library: Library::new(filename).unwrap(),
        }
    }
}

impl DLLib {
    fn create_string<F>(&self, lambda: F) -> String
    where
        F: FnOnce() -> *const c_char,
    {
        unsafe {
            let free_string: Symbol<StringConsumer> = self.library.get(b"FreeString").unwrap();

            let raw_str = lambda();
            let c_str = CStr::from_ptr(raw_str).to_str().unwrap().to_owned();
            free_string(raw_str);

            c_str
        }
    }

    pub fn get_string(&self) -> String {
        unsafe {
            let get_string: Symbol<StringSupplier> = self.library.get(b"GetString").unwrap();
            self.create_string(|| get_string())
        }
    }
}
