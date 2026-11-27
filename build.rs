//! Embeds the application icon and version metadata into the Windows
//! executable and statically links the VC runtime so the exe runs on a bare
//! Windows install (fresh VMs, debloated images) without redistributables.

fn main() {
    println!("cargo:rerun-if-changed=assets/favicon.ico");

    // `cfg!(windows)` would describe the *host*; we care about the *target*.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    static_vcruntime::metabuild();

    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/favicon.ico")
        .set("ProductName", "Windows Configurator")
        .set("FileDescription", "Windows Configurator");
    if let Err(err) = res.compile() {
        panic!("failed to embed Windows resources: {err}");
    }
}
