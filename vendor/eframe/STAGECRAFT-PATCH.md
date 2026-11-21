eframe 0.37.0 from crates.io (MIT OR Apache-2.0, see its Cargo.toml) with one change for Windows 7:
src/native/file_storage.rs looks up CoTaskMemFree in ole32.dll at run time
(a by-name import resolves to combase.dll, which Windows 7 does not have).
