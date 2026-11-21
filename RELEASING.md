# Publishing a new version of Stagecraft 3D

**Pushing with GitHub Desktop publishes the release.** The finished Windows and Linux programs
live in the repository under `releases/v<version>/`. When GitHub Desktop pushes a new folder
there, GitHub creates the release from those ready-made files (nothing is compiled for Windows
or Linux), then builds the Mac version and adds it to the same release.

## Publish a version with GitHub Desktop

1. Open **GitHub Desktop**. Top left, *Current repository* › choose **StageCraft-3D**.
   (Not listed? *File › Add local repository…* and pick `Documents\GitHub\StageCraft-3D`.)
2. If files are listed on the left, type a summary such as `Release v0.4.0` and click
   **Commit to main**. If the list is empty, the release is already committed: go to step 3.
3. Click **Push origin** (top bar). The upload takes a few minutes (about 230 MB the first time).
4. That's it. On GitHub (*Repository* › **View on GitHub**, then **Releases**) the release
   **Stagecraft 3D v0.4.0** appears within a minute or two with all 25 Windows and Linux files.
   The Mac `.dmg` and `.zip` are added about 5 minutes later. Progress is shown under **Actions**.

## What's in a release

- Windows 10/11 64-bit, 32-bit and ARM64, plus Windows 7/8/8.1 64-bit and 32-bit:
  each as `.msi` installer, portable `.zip` and plain `.exe` (15 files).
- Linux for PCs (x86_64) and ARM (aarch64): AppImage, `.tar.gz`, `.deb`, `.rpm` and Arch
  `.pkg.tar.zst` (10 files).
- macOS universal `.dmg` and `.zip` (added by GitHub).

## The next version

1. Change `version` in `Cargo.toml` and in `packaging/windows/stagecraft.rc`, and add a section
   to `CHANGELOG.md`.
2. Build everything: `powershell -ExecutionPolicy Bypass -File packaging\build-all.ps1`.
   It builds the Windows and Linux files and puts them in `releases\v<new version>\`.
3. Commit and push in GitHub Desktop (steps 1–3 above).

Each version adds about 230 MB to the repository. If it ever gets too big, delete old
`releases/v…` folders; the published releases on GitHub keep their files.

## Other ways

- Double-click **`Publish release.cmd`**: creates the release and uploads the files from `dist`
  using the GitHub CLI (no files added to the repository).
- On the GitHub website: *Releases* › *Draft a new release*, drag the files in, *Publish*.
- `Add files to latest release.cmd` attaches the files to a release that already exists.
