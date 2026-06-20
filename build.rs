use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if target.contains("windows") {
        // Windows-specific build configuration
        println!("cargo:rustc-link-lib=setupapi");
        println!("cargo:rustc-link-lib=cfgmgr32");
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=shell32");

        // Embed icon and version info
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icons/app.ico");
        res.set("FileDescription", "Sidozdev - USB Bootable Media Creator");
        res.set("ProductName", "Sidozdev");
        res.set("ProductVersion", "1.0.0");
        res.set("FileVersion", "1.0.0.0");
        res.set("LegalCopyright", "Copyright (c) 2024 Sidozdev Team");
        res.set("OriginalFilename", "sidozdev.exe");

        if let Err(e) = res.compile() {
            eprintln!("Warning: Could not compile Windows resources: {}", e);
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/");
}
