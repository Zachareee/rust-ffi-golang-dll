use std::error::Error;
use std::ffi::{CStr, c_char};

use libloading::{Library, Symbol};

type StringSupplier = unsafe extern "C" fn() -> *const c_char;
type StringConsumer = unsafe extern "C" fn(*const c_char);

fn main() -> Result<(), Box<dyn Error>> {
    let lib_path = "./test.dll";
    unsafe {
        let lib = Library::new(lib_path)?;
        let get_string: Symbol<StringSupplier> = lib.get(b"GetString")?;
        let free_string: Symbol<StringConsumer> = lib.get(b"FreeString")?;

        let raw_str = get_string();
        let c_str = CStr::from_ptr(raw_str);
        println!("{}", c_str.to_str()?);

        free_string(raw_str);
    };
    Ok(())
}
