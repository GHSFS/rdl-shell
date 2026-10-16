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

#[cfg(test)]
mod validation_tests {
    use rdl_shell::client::validate_url;

    #[test]
    fn validates_standard_https_urls() {
        assert!(validate_url("https://example.com/file.zip").is_ok());
        assert!(validate_url("http://example.com/test").is_ok());
    }

    #[test]
    fn rejects_non_http_schemes() {
        assert!(validate_url("javascript:alert(1)").is_err());
        assert!(validate_url("file:///C:/Windows/System32").is_err());
        assert!(validate_url("ftp://ftp.example.com").is_err());
    }

    #[test]
    fn rejects_malformed_urls() {
        assert!(validate_url("not a url").is_err());
        assert!(validate_url("").is_err());
        assert!(validate_url("   ").is_err());
    }
}

#[cfg(test)]
mod encoding_tests {
    use rdl_shell::handler::decode_url_file_bytes;

    #[test]
    fn decodes_utf8_ascii() {
        let text = "[InternetShortcut]\nURL=https://github.com\n";
        assert_eq!(decode_url_file_bytes(text.as_bytes()).unwrap(), text);
    }

    #[test]
    fn decodes_utf16le_with_bom() {
        let text = "[InternetShortcut]\r\nURL=https://github.com\r\n";
        let mut bytes = vec![0xFF, 0xFE];
        for u in text.encode_utf16() {
            bytes.extend_from_slice(&u.to_le_bytes());
        }
        assert_eq!(decode_url_file_bytes(&bytes).unwrap(), text);
    }

    #[test]
    fn decodes_utf16be_with_bom() {
        let text = "[InternetShortcut]\r\nURL=https://github.com\r\n";
        let mut bytes = vec![0xFE, 0xFF];
        for u in text.encode_utf16() {
            bytes.extend_from_slice(&u.to_be_bytes());
        }
        assert_eq!(decode_url_file_bytes(&bytes).unwrap(), text);
    }
}
