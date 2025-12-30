mod dllib;

fn main() {
    let lib_path = "./test.dll";
    let library = dllib::load(lib_path);

    println!("{}", library.get_string());
}
