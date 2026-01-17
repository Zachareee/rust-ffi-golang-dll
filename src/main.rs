use std::{error::Error, fmt::Debug, fs, time::SystemTime};

mod dllib;

const CREDS_PATH: &str = "./credentials.json";

fn main() -> Result<(), Box<dyn Error>> {
    let lib_path = "./test.dll";

    let mut creds = fs::read_to_string(CREDS_PATH).unwrap_or_default();

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
                    let _ = fs::write(&CREDS_PATH, &token);
                    creds = token;
                }
                Err(err) => println!("{err}"),
            }
        } else {
            println!("Authenticated")
        }

        // if let Err(e) = library.upload(
        //     &creds,
        //     "third test",
        //     SystemTime::now()
        //         .duration_since(SystemTime::UNIX_EPOCH)
        //         .unwrap_or_default()
        //         .as_secs(),
        //     "LETS GO".into(),
        // ) {
        //     println!("Error occured: {e}");
        // }

        match library.read_cloud(&creds) {
            Ok(v) => {
                println!("Vector: {v:?}");
            }
            Err(e) => println!("Error: {e}"),
        }

        library.download(&creds, "third test");
    };
    Ok(())
}
