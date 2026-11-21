//! Builds the Stagecraft 3D Linux release files from an already-built Linux binary,
//! on any operating system (no Linux tools needed). For <arch> x86_64 or aarch64:
//!   Stagecraft3D-<ver>-linux-<arch>.tar.gz     portable folder, any distribution
//!   Stagecraft3D-<ver>-linux-<arch>.AppImage   single file, any distribution
//!   stagecraft-3d_<ver>_<amd64|arm64>.deb      Debian / Ubuntu / Mint / Pop!_OS / Raspberry Pi OS
//!   stagecraft-3d-<ver>-1.<arch>.rpm           Fedora / openSUSE / RHEL / Rocky / Alma
//!   stagecraft-3d-<ver>-1-<arch>.pkg.tar.zst   Arch / Manjaro / EndeavourOS (Arch Linux ARM)
//!
//! Usage: linux-packager --bin <linux binary> --project <project dir> --version <x.y.z>
//!                       --out <dir> [--arch x86_64|aarch64] [--runtime <AppImage type2 runtime>]

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

type R<T> = Result<T, Box<dyn Error>>;

const NAME: &str = "stagecraft-3d";
const SUMMARY: &str = "Plan and preview theatre sets in 3D";
const DESCRIPTION: &str = "Stagecraft 3D lets directors, stage managers and set designers plan and \
preview sets in real venue presets: furniture and scenery, flying bars, revolve and lifts, \
lighting with beam previews, sightline warnings, scenes and PNG/JSON export.";
const URL: &str = "https://github.com/Arrestis/StageCraft-3D";
const LICENSE: &str = "GPL-3.0-only";

/// One file in the installed tree.
struct Entry {
    path: String, // absolute, e.g. /usr/bin/stagecraft-3d
    data: Vec<u8>,
    mode: u32,
}

struct Args {
    bin: PathBuf,
    project: PathBuf,
    version: String,
    out: PathBuf,
    arch: String,
    runtime: Option<PathBuf>,
}

fn args() -> R<Args> {
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let need = |k: &str| get(k).ok_or_else(|| format!("missing {k}"));
    let arch = get("--arch").unwrap_or_else(|| "x86_64".into());
    if arch != "x86_64" && arch != "aarch64" {
        return Err(format!("unknown --arch {arch} (use x86_64 or aarch64)").into());
    }
    Ok(Args {
        bin: need("--bin")?.into(),
        project: need("--project")?.into(),
        version: need("--version")?,
        out: need("--out")?.into(),
        arch,
        runtime: get("--runtime").map(PathBuf::from),
    })
}

fn deb_arch(arch: &str) -> &'static str {
    if arch == "aarch64" { "arm64" } else { "amd64" }
}

fn tree(bin: &[u8], project: &Path) -> R<Vec<Entry>> {
    Ok(vec![
        Entry { path: format!("/usr/bin/{NAME}"), data: bin.to_vec(), mode: 0o755 },
        Entry {
            path: format!("/usr/share/applications/{NAME}.desktop"),
            data: fs::read(project.join("packaging/linux/stagecraft-3d.desktop"))?,
            mode: 0o644,
        },
        Entry {
            path: format!("/usr/share/icons/hicolor/512x512/apps/{NAME}.png"),
            data: fs::read(project.join("assets/icon.png"))?,
            mode: 0o644,
        },
        Entry {
            path: format!("/usr/share/icons/hicolor/256x256/apps/{NAME}.png"),
            data: fs::read(project.join("assets/icon-256.png"))?,
            mode: 0o644,
        },
        Entry {
            path: format!("/usr/share/doc/{NAME}/README.md"),
            data: fs::read(project.join("README.md"))?,
            mode: 0o644,
        },
        Entry {
            path: format!("/usr/share/licenses/{NAME}/LICENSE"),
            data: fs::read(project.join("LICENSE"))?,
            mode: 0o644,
        },
    ])
}

fn mtime() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn tar_header(path: &str, size: u64, mode: u32, dir: bool) -> R<tar::Header> {
    let mut h = tar::Header::new_gnu();
    h.set_path(path)?;
    h.set_size(size);
    h.set_mode(mode);
    h.set_uid(0);
    h.set_gid(0);
    h.set_username("root")?;
    h.set_groupname("root")?;
    h.set_mtime(mtime());
    h.set_entry_type(if dir { tar::EntryType::Directory } else { tar::EntryType::Regular });
    h.set_cksum();
    Ok(h)
}

/// Uncompressed tar of `files`, each path prefixed with `prefix` (e.g. "./"), with parent dirs.
fn tar_bytes(files: &[(String, &[u8], u32)], prefix: &str, with_root: bool) -> R<Vec<u8>> {
    let mut t = tar::Builder::new(Vec::new());
    let mut dirs = BTreeSet::new();
    for (p, _, _) in files {
        let mut cur = String::new();
        let parts: Vec<&str> = p.split('/').filter(|s| !s.is_empty()).collect();
        for part in &parts[..parts.len() - 1] {
            cur.push_str(part);
            cur.push('/');
            dirs.insert(cur.clone());
        }
    }
    // Package metadata (.PKGINFO, .MTREE) goes first, as makepkg does.
    let is_meta = |p: &str| p.starts_with('.') && !p.contains('/');
    for (p, data, mode) in files.iter().filter(|f| is_meta(&f.0)) {
        let h = tar_header(&format!("{prefix}{p}"), data.len() as u64, *mode, false)?;
        t.append(&h, *data)?;
    }
    if with_root {
        let h = tar_header(prefix, 0, 0o755, true)?;
        t.append(&h, std::io::empty())?;
    }
    for d in &dirs {
        let h = tar_header(&format!("{prefix}{d}"), 0, 0o755, true)?;
        t.append(&h, std::io::empty())?;
    }
    for (p, data, mode) in files.iter().filter(|f| !is_meta(&f.0)) {
        let h = tar_header(&format!("{prefix}{}", p.trim_start_matches('/')), data.len() as u64, *mode, false)?;
        t.append(&h, *data)?;
    }
    Ok(t.into_inner()?)
}

fn gzip(data: &[u8]) -> R<Vec<u8>> {
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    gz.write_all(data)?;
    Ok(gz.finish()?)
}

fn tar_gz(files: &[(String, &[u8], u32)], prefix: &str, with_root: bool) -> R<Vec<u8>> {
    gzip(&tar_bytes(files, prefix, with_root)?)
}

fn build_tar_gz(a: &Args, entries: &[Entry]) -> R<PathBuf> {
    let folder = format!("Stagecraft3D-{}-linux-{}/", a.version, a.arch);
    let desktop = fs::read(a.project.join("packaging/linux/stagecraft-3d.desktop"))?;
    let icon = fs::read(a.project.join("assets/icon.png"))?;
    let readme = fs::read(a.project.join("README.md"))?;
    let license = fs::read(a.project.join("LICENSE"))?;
    let files: Vec<(String, &[u8], u32)> = vec![
        (NAME.to_string(), entries[0].data.as_slice(), 0o755),
        (format!("{NAME}.desktop"), desktop.as_slice(), 0o644),
        (format!("{NAME}.png"), icon.as_slice(), 0o644),
        ("README.md".to_string(), readme.as_slice(), 0o644),
        ("LICENSE".to_string(), license.as_slice(), 0o644),
    ];
    let path = a.out.join(format!("Stagecraft3D-{}-linux-{}.tar.gz", a.version, a.arch));
    fs::write(&path, tar_gz(&files, &folder, true)?)?;
    Ok(path)
}

fn build_deb(a: &Args, entries: &[Entry]) -> R<PathBuf> {
    use md5::{Digest, Md5};
    let installed_kb: usize = entries.iter().map(|e| e.data.len()).sum::<usize>() / 1024 + 1;
    let darch = deb_arch(&a.arch);
    let control = format!(
        "Package: {NAME}\nVersion: {v}-1\nArchitecture: {darch}\nMaintainer: Stagecraft 3D <stagecraft3d@users.noreply.github.com>\n\
Installed-Size: {installed_kb}\nDepends: libc6 (>= 2.28)\nRecommends: libxkbcommon0, libxkbcommon-x11-0, libvulkan1, libegl1, libgl1\n\
Section: graphics\nPriority: optional\nHomepage: {URL}\nDescription: {SUMMARY}\n {DESCRIPTION}\n",
        v = a.version
    );
    let mut md5sums = String::new();
    for e in entries {
        let d = Md5::digest(&e.data);
        let hex: String = d.iter().map(|b| format!("{b:02x}")).collect();
        md5sums.push_str(&format!("{hex}  {}\n", e.path.trim_start_matches('/')));
    }
    let control_tar = tar_gz(
        &[("control".into(), control.as_bytes(), 0o644), ("md5sums".into(), md5sums.as_bytes(), 0o644)],
        "./",
        true,
    )?;
    let data_files: Vec<(String, &[u8], u32)> = entries.iter().map(|e| (e.path.clone(), e.data.as_slice(), e.mode)).collect();
    let data_tar = tar_gz(&data_files, "./", true)?;

    let path = a.out.join(format!("{NAME}_{}_{darch}.deb", a.version));
    let f = fs::File::create(&path)?;
    let mut ar = ar::Builder::new(f);
    for (name, data) in [
        ("debian-binary", b"2.0\n".to_vec()),
        ("control.tar.gz", control_tar),
        ("data.tar.gz", data_tar),
    ] {
        let mut h = ar::Header::new(name.as_bytes().to_vec(), data.len() as u64);
        h.set_mode(0o100644);
        h.set_mtime(mtime());
        h.set_uid(0);
        h.set_gid(0);
        ar.append(&h, data.as_slice())?;
    }
    Ok(path)
}

fn build_rpm(a: &Args, entries: &[Entry]) -> R<PathBuf> {
    let cfg = rpm::BuildConfig::default().compression(rpm::CompressionType::Gzip);
    let mut b = rpm::PackageBuilder::new(NAME, &a.version, LICENSE, &a.arch, SUMMARY);
    b.using_config(cfg).release("1").url(URL).vendor("Stagecraft 3D").description(DESCRIPTION);
    for e in entries {
        b.with_file_contents(e.data.clone(), rpm::FileOptions::new(e.path.clone()).permissions(e.mode as u16))?;
    }
    let pkg = b.build()?;
    let path = a.out.join(format!("{NAME}-{}-1.{}.rpm", a.version, a.arch));
    pkg.write_file(&path)?;
    Ok(path)
}

/// Arch Linux package: zstd-compressed tar with .PKGINFO and .MTREE, as made by makepkg.
fn build_pacman(a: &Args, entries: &[Entry]) -> R<PathBuf> {
    use md5::{Digest, Md5};
    let now = mtime();
    let size: usize = entries.iter().map(|e| e.data.len()).sum();
    let pkginfo = format!(
        "# Generated by Stagecraft 3D linux-packager\npkgname = {NAME}\npkgbase = {NAME}\npkgver = {v}-1\n\
pkgdesc = {SUMMARY}\nurl = {URL}\nbuilddate = {now}\npackager = Stagecraft 3D\nsize = {size}\narch = {arch}\n\
license = {LICENSE}\ndepend = glibc\ndepend = libxkbcommon\noptdepend = vulkan-icd-loader: faster drawing\n",
        v = a.version,
        arch = a.arch
    );

    // .MTREE: one line per directory and file (gzip-compressed mtree text).
    let mut mtree = String::from("#mtree\n/set type=file uid=0 gid=0 mode=644\n");
    mtree.push_str(&format!("./.PKGINFO time={now}.0 size={}\n", pkginfo.len()));
    let mut dirs = BTreeSet::new();
    for e in entries {
        let parts: Vec<&str> = e.path.trim_start_matches('/').split('/').collect();
        let mut cur = String::new();
        for part in &parts[..parts.len() - 1] {
            if !cur.is_empty() {
                cur.push('/');
            }
            cur.push_str(part);
            dirs.insert(cur.clone());
        }
    }
    for d in &dirs {
        mtree.push_str(&format!("./{d} time={now}.0 mode=755 type=dir\n"));
    }
    for e in entries {
        let hex: String = Md5::digest(&e.data).iter().map(|b| format!("{b:02x}")).collect();
        mtree.push_str(&format!(
            "./{} time={now}.0 mode={:o} size={} md5digest={hex}\n",
            e.path.trim_start_matches('/'),
            e.mode,
            e.data.len()
        ));
    }
    let mtree_gz = gzip(mtree.as_bytes())?;

    let mut files: Vec<(String, &[u8], u32)> = vec![
        (".PKGINFO".into(), pkginfo.as_bytes(), 0o644),
        (".MTREE".into(), mtree_gz.as_slice(), 0o644),
    ];
    for e in entries {
        files.push((e.path.trim_start_matches('/').to_string(), e.data.as_slice(), e.mode));
    }
    let tar = tar_bytes(&files, "", false)?;
    let zst = zstd::encode_all(Cursor::new(tar), 19)?;
    let path = a.out.join(format!("{NAME}-{}-1-{}.pkg.tar.zst", a.version, a.arch));
    fs::write(&path, zst)?;
    Ok(path)
}

fn build_appimage(a: &Args, entries: &[Entry], runtime: &Path) -> R<PathBuf> {
    use backhand::{compression::Compressor, FilesystemCompressor, FilesystemWriter, NodeHeader};
    let t = mtime() as u32;
    let dir_h = NodeHeader::new(0o755, 0, 0, t);
    let file_h = |mode: u32| NodeHeader::new(mode as u16, 0, 0, t);
    let apprun = format!(
        "#!/bin/sh\nHERE=\"$(dirname \"$(readlink -f \"$0\")\")\"\nexec \"$HERE/usr/bin/{NAME}\" \"$@\"\n"
    );
    let desktop = fs::read(a.project.join("packaging/linux/stagecraft-3d.desktop"))?;
    let icon = fs::read(a.project.join("assets/icon.png"))?;

    let mut fs_w = FilesystemWriter::default();
    fs_w.set_compressor(FilesystemCompressor::new(Compressor::Zstd, None)?);
    fs_w.set_time(t);
    fs_w.set_root_mode(0o755);
    fs_w.push_file(Cursor::new(apprun.into_bytes()), "AppRun", file_h(0o755))?;
    fs_w.push_file(Cursor::new(desktop), format!("{NAME}.desktop"), file_h(0o644))?;
    fs_w.push_file(Cursor::new(icon.clone()), format!("{NAME}.png"), file_h(0o644))?;
    fs_w.push_file(Cursor::new(icon), ".DirIcon", file_h(0o644))?;
    let mut made = BTreeSet::new();
    for e in entries {
        let rel = e.path.trim_start_matches('/');
        let parts: Vec<&str> = rel.split('/').collect();
        let mut cur = String::new();
        for part in &parts[..parts.len() - 1] {
            if !cur.is_empty() {
                cur.push('/');
            }
            cur.push_str(part);
            if made.insert(cur.clone()) {
                fs_w.push_dir(&cur, dir_h)?;
            }
        }
        fs_w.push_file(Cursor::new(e.data.clone()), rel, file_h(e.mode))?;
    }

    let rt = fs::read(runtime)?;
    let path = a.out.join(format!("Stagecraft3D-{}-linux-{}.AppImage", a.version, a.arch));
    let mut f = fs::File::create(&path)?;
    f.write_all(&rt)?;
    fs_w.write_with_offset(&mut f, rt.len() as u64)?;
    drop(f);

    // Self-check: read the image back and make sure the program is inside and intact.
    let rd = std::io::BufReader::new(fs::File::open(&path)?);
    let fsr = backhand::FilesystemReader::from_reader_with_offset(rd, rt.len() as u64)?;
    let mut found = false;
    for node in fsr.files() {
        if let backhand::InnerNode::File(file) = &node.inner {
            if node.fullpath.ends_with(format!("usr/bin/{NAME}")) {
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut fsr.file(file).reader(), &mut buf)?;
                found = buf == entries[0].data && node.header.permissions == 0o755;
            }
        }
    }
    if !found {
        return Err("AppImage self-check failed: program missing or damaged".into());
    }
    Ok(path)
}

fn main() -> R<()> {
    let a = args()?;
    fs::create_dir_all(&a.out)?;
    let bin = fs::read(&a.bin)?;
    if bin.get(..4) != Some(b"\x7fELF".as_slice()) {
        return Err(format!("{} is not a Linux (ELF) program", a.bin.display()).into());
    }
    // ELF e_machine: 0x3E = x86_64, 0xB7 = aarch64.
    let machine = u16::from_le_bytes([bin[18], bin[19]]);
    let expect = if a.arch == "aarch64" { 0xB7 } else { 0x3E };
    if machine != expect {
        return Err(format!("{} is not a {} program (ELF machine {machine:#x})", a.bin.display(), a.arch).into());
    }
    let entries = tree(&bin, &a.project)?;
    println!("{}", build_tar_gz(&a, &entries)?.display());
    println!("{}", build_deb(&a, &entries)?.display());
    println!("{}", build_rpm(&a, &entries)?.display());
    println!("{}", build_pacman(&a, &entries)?.display());
    if let Some(rt) = &a.runtime {
        println!("{}", build_appimage(&a, &entries, rt)?.display());
    }
    Ok(())
}
