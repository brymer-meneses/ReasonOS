
use std::error::Error;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn Error>> {
    for entry in WalkDir::new("src") {
        let entry = entry.unwrap();
        let path = entry.path();

        let object_os = path.file_name().expect("Failed to get file name");
        let object_file = object_os.to_str().expect("Invalid UTF-8 for file name");

        match path.extension() {
            Some(ext) if ext.eq("asm") => {
                let mut build = nasm_rs::Build::new();

                build
                    .file(&path)
                    .flag("-felf64")
                    .target("x86_64-unknown-none")
                    .compile(object_file)
                    .expect(&format!("failed to compile assembly {:?}", path));

                println!("cargo:rustc-link-lib=static={}", object_file);
                println!("cargo:rerun-if-changed={}", path.display());
            }

            _ => (),
        }

    }

    println!("cargo:rerun-if-changed=.cargo/kernel.ld");

    Ok(())
}
