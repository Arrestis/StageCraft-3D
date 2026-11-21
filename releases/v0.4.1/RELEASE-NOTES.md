### What's new in 0.4.1

- **TAS Hoskins Centre** rebuilt from its technical specifications: 12 m x 7.65 m proscenium, four lighting trusses (two motorised), movable masking tabs, a 7.5 m x 2.5 m pit and 230 seats with East and West side galleries.
- **Capitol Theatre Tamworth** now hangs its real 17-line hanging plot (gold house curtain, false proscenium, legs and borders, four LX bars, black tabs, white cyc), with 402 seats, the cross aisle, perches and box booms.
- **Genesian Theatre** Rozelle (teal and gold, steep seating with aisle stairs both sides) and the old Kent Street church (oak panelling, timber roof, rear gallery) now look like the real rooms; **Lazenby Hall** gets its timber hall look, side gallery and screens.
- New Venue settings: room look, side and rear galleries, aisle stairs, cross aisle, screens. The Director view now shows the whole stage opening.

Ready-to-run versions of **Stagecraft 3D 0.4.1**: nothing to compile. Pick the file for your computer.

### Windows

| Your computer | Installer (recommended) | Portable (no install) | Program only |
| --- | --- | --- | --- |
| Windows 10 / 11, 64-bit (most PCs) | `Stagecraft3D-0.4.1-windows-x64.msi` | `…-windows-x64-portable.zip` | `…-windows-x64.exe` |
| Windows 10 / 11, 32-bit | `Stagecraft3D-0.4.1-windows-x86.msi` | `…-windows-x86-portable.zip` | `…-windows-x86.exe` |
| Windows 10 / 11 on ARM (Snapdragon, Surface Pro X, Copilot+ PCs) | `Stagecraft3D-0.4.1-windows-arm64.msi` | `…-windows-arm64-portable.zip` | `…-windows-arm64.exe` |
| Windows 7, 8 or 8.1, 64-bit | `Stagecraft3D-0.4.1-windows7-x64.msi` | `…-windows7-x64-portable.zip` | `…-windows7-x64.exe` |
| Windows 7, 8 or 8.1, 32-bit | `Stagecraft3D-0.4.1-windows7-x86.msi` | `…-windows7-x86-portable.zip` | `…-windows7-x86.exe` |

Not sure if your Windows is 64-bit? *Settings › System › About › System type* (Windows 7: *Control Panel › System*).
The Windows 7/8 versions draw with OpenGL, so the graphics card driver needs to be installed.

### Mac

| Your computer | Download |
| --- | --- |
| Any Mac, Apple Silicon (M1–M4) or Intel, macOS 11 or later | `Stagecraft3D-0.4.1-macos-universal.dmg` (open it, drag **Stagecraft 3D** to Applications) or `…-macos-universal.zip` |

The Mac files are added by GitHub a few minutes after the release appears. First launch: right-click the app › **Open**.

### Linux (PC = x86_64, ARM = aarch64 such as Raspberry Pi 4/5)

| Distribution | PC (x86_64) | ARM (aarch64) | Install with |
| --- | --- | --- | --- |
| Any distribution | `Stagecraft3D-0.4.1-linux-x86_64.AppImage` | `…-linux-aarch64.AppImage` | `chmod +x` the file, then run it |
| Any distribution | `Stagecraft3D-0.4.1-linux-x86_64.tar.gz` | `…-linux-aarch64.tar.gz` | unpack, run `./stagecraft-3d` |
| Debian, Ubuntu, Mint, Pop!_OS, Raspberry Pi OS | `stagecraft-3d_0.4.1_amd64.deb` | `stagecraft-3d_0.4.1_arm64.deb` | `sudo apt install ./stagecraft-3d_…deb` |
| Fedora, openSUSE, RHEL, Rocky, Alma | `stagecraft-3d-0.4.1-1.x86_64.rpm` | `stagecraft-3d-0.4.1-1.aarch64.rpm` | `sudo dnf install ./stagecraft-3d-…rpm` (openSUSE: `sudo zypper install`) |
| Arch, Manjaro, EndeavourOS | `stagecraft-3d-0.4.1-1-x86_64.pkg.tar.zst` | `stagecraft-3d-0.4.1-1-aarch64.pkg.tar.zst` | `sudo pacman -U stagecraft-3d-…pkg.tar.zst` |

Linux versions need glibc 2.28 or newer (Ubuntu 20.04+, Debian 10+, Fedora 29+ and current Arch).

---
Stagecraft 3D is free software under the GNU GPL v3. The files aren't code-signed: Windows may say
*More info › Run anyway* the first time.
