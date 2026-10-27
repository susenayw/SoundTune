fn main() {
    let status = std::process::Command::new("windres").args(["assets/app.rc", "-O", "coff", "-o", "icon.o"]).status().expect("windres is required to embed the Windows icon");
    assert!(status.success(), "could not compile application icon");
    println!("cargo:rustc-link-arg=icon.o");
    println!("cargo:rerun-if-changed=assets/soundtune.ico");
}
