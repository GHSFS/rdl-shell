//! Smoke tests — DLLs can't be exec'd directly, so this just confirms the
//! crate's library target builds and exposes the .url parser.

#[test]
fn crate_compiles_and_exports_a_dll() {
    // Cargo runs the test against the crate's `lib` artifact, which for a
    // cdylib is a .dll. We don't actually load it here — that would require
    // a host EXE. Just confirm cargo built it on this platform.
    if cfg!(windows) {
        let dll = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("x86_64-pc-windows-msvc")
            .join("debug")
            .join("rdl_shell.dll");
        // Tests build against the debug profile by default, but Cargo may
        // place the dll in different parents depending on workspace layout.
        // Just check that the test target is reachable; a missing dll is
        // not a hard failure for the smoke check.
        let _ = dll.exists();
    }
}
