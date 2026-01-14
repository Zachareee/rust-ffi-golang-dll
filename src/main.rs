use std::{error::Error, fs};

mod dllib;

const CREDS_PATH: &str = "./credentials.json";

fn main() -> Result<(), Box<dyn Error>> {
    let lib_path = "./test.dll";

    let creds = fs::read_to_string(CREDS_PATH).unwrap_or_default();

    unsafe {
        let library = dllib::load(lib_path)?;

        let tup = library.validate(&creds, "http://localhost");

        if let Some(err) = tup.1 {
            println!("Error found: {err}");

            println!("{}", tup.0.unwrap());
            let code = {
                use std::io::stdin;
                let mut s = String::new();
                stdin().read_line(&mut s)?;
                s
            };

            let e = code
                .strip_suffix("\r\n")
                .or(code.strip_suffix("\n"))
                .unwrap();

            match library.extract_credentials(&e) {
                Ok(token) => {
                    println!("Authenticated");
                    let _ = fs::write(&CREDS_PATH, token);
                }
                Err(err) => println!("{err}"),
            }
        } else {
            println!("Authenticated")
        }
    };
    Ok(())
}
