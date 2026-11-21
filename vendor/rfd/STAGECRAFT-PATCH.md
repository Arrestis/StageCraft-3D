rfd 0.15.4 from crates.io (MIT, see LICENSE) with one change for Windows 7:
src/backend/win_cid/file_dialog/com.rs looks up CoTaskMemFree in ole32.dll at run time
(a by-name import resolves to combase.dll, which Windows 7 does not have).
