use std::error::Error;

mod dllib;

fn main() -> Result<(), Box<dyn Error>> {
    let lib_path = "./test.dll";
    unsafe {
        let library = dllib::load(lib_path)?;

        println!("{}", library.get_string()?);
        println!("{:?}", library.get_str_arr()?);
        library.print_string(c"String from Rust to Go").unwrap();
    };
    Ok(())
}
