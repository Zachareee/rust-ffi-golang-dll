use std::{
    error::Error,
    ffi::{CStr, c_char},
};

use libloading::{AsFilename, Library};

type DLLString = *const c_char;

type StringSupplier = unsafe extern "C" fn() -> DLLString;
type StringConsumer = unsafe extern "C" fn(DLLString);

#[repr(C)]
pub struct DLLStruct {
    count: i32,
    strings: *const DLLString,
}

type StructSupplier = unsafe extern "C" fn() -> DLLStruct;
type SideEffect = extern "C" fn() -> ();
type DLLResult<T> = Result<T, Box<dyn Error>>;

pub struct DLLib {
    library: Library,
}

pub unsafe fn load(filename: impl AsFilename) -> DLLResult<DLLib> {
    Ok(DLLib {
        library: unsafe { Library::new(filename)? },
    })
}

impl DLLib {
    unsafe fn create_string<F>(&self, lambda: F) -> DLLResult<String>
    where
        F: FnOnce() -> *const c_char,
    {
        let raw_str = lambda();
        let c_str = unsafe { CStr::from_ptr(raw_str) }.to_str()?.to_owned();
        unsafe {
            let free_string = self.library.get::<StringConsumer>(b"FreeString")?;
            free_string(raw_str);
        }

        Ok(c_str)
    }

    pub unsafe fn get_string(&self) -> DLLResult<String> {
        unsafe {
            let get_string = self.library.get::<StringSupplier>(b"GetString")?;
            self.create_string(|| get_string())
        }
    }

    pub unsafe fn get_str_arr(&self) -> DLLResult<Vec<String>> {
        let mut v: Vec<String> = Vec::new();

        unsafe {
            let get_struct = self.library.get::<StructSupplier>(b"GetStruct")?;
            let s = get_struct();
            for i in 0..s.count {
                v.push(self.create_string(|| *s.strings.offset(i as isize))?);
            }
            self.library.get::<SideEffect>(b"FreeStruct")?();
        }

        Ok(v)
    }
}
