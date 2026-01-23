use std::{error::Error, fs, time::SystemTime};

mod dllib;

const CREDS_PATH: &str = "./credentials.json";

fn main() -> Result<(), Box<dyn Error>> {
    let lib_path = "./test.dll";

    let mut creds = fs::read_to_string(CREDS_PATH).unwrap_or_default();

    let library = unsafe { dllib::load(lib_path) }?;

    let tup = unsafe { library.validate(&creds, "http://localhost") };

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

        match unsafe { library.extract_credentials(&e) } {
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

    let info = unsafe { library.info() };

    println!("{:?}", info);

    let filename = "upload.zip";

    if let Err(e) = unsafe {
        library.upload(
            &creds,
            &filename,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            fs::read(&filename).unwrap(),
        )
    } {
        println!("Error occured: {e}");
    }

    match unsafe { library.read_cloud(&creds) } {
        Ok(v) => {
            println!("Vector: {v:?}");
        }
        Err(e) => println!("Error: {e}"),
    }

    match unsafe { library.download(&creds, &filename) } {
        Err(err) => println!("{err}"),
        Ok(v) => fs::write("downloaded.zip", v).unwrap(),
    };
    Ok(())
}
