//! Embeds the Win32 resource script (manifest + version) and tells the linker
//! to use our `.def` file so the four COM exports are visible by name to
//! `regsvr32.exe`.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("resources/app.rc", embed_resource::NONE);

        let def = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources/rdl-shell.def");
        println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def.display());

        println!("cargo:rerun-if-changed=resources/app.rc");
        println!("cargo:rerun-if-changed=resources/app.manifest");
        println!("cargo:rerun-if-changed=resources/rdl-shell.def");
    }
}
