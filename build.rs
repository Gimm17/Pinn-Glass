use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.ico");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let res_path = format!("{}/app.res", out_dir);

    let windres_candidates = ["windres", r"C:\Users\Lyvn\w64devkit\bin\windres.exe"];
    let mut compiled = false;

    for &windres in &windres_candidates {
        let status = Command::new(windres)
            .args(&["-i", "app.rc", "-O", "coff", "-o", &res_path])
            .status();

        if let Ok(s) = status {
            if s.success() {
                println!("cargo:rustc-link-arg={}", res_path);
                println!("cargo:warning=Successfully embedded app.ico into executable!");
                compiled = true;
                break;
            }
        }
    }

    if !compiled {
        panic!("Failed to compile Windows resource with windres!");
    }
}
