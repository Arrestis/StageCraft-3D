# Stagecraft 3D

*by Doug Rumble*

A simple desktop app for planning and previewing theatre sets in 3D. It's for directors,
stage managers and set designers: pick a real venue, drop in furniture and scenery,
fly bars in and out, light the scene and check sightlines before anything is built.
Free and open-source software under the GNU GPL v3.

## Features

- **Venue presets** – Armidale Playhouse, TAS Hoskins Centre, Lazenby Hall (UNE), Capitol Theatre
  Tamworth, Genesian Theatre (Rozelle, plus its former Kent Street church), a large West End /
  Broadway-style proscenium house, a flexible black box, and a Custom Stage. Each one has
  editable opening, depth, wings, fly grid / dead-hung pipes, rake, orchestra pit / pit lift,
  revolve and stage lift.
  - The *Armidale Playhouse* preset follows the venue's as-built floor plan: an 8.3 m x 3.75 m
    open end stage on a 450 mm riser, a 3.2 m ceiling sloping to 2.3 m on stage left, about
    120 raked seats with a house-right aisle, and its real lighting positions.
  - *Capitol Theatre Tamworth* uses the published technical specification, including its 17-line
    hanging plot, drapery sizes and lighting positions.
  - *TAS Hoskins Centre* follows its technical dossier: 12 m x 7.65 m proscenium, four lighting
    trusses, movable masking tabs, a 7.5 m x 2.5 m pit and 230 seats with side galleries.
  - *Genesian* (Rozelle and Kent Street) and *Lazenby Hall* take their room finishes, galleries,
    aisles and screens from venue photos.
  - Where no published data was found, the figures are estimates and the app says so.
- **3D stage** – deck with grid and metre/foot markings, proscenium or open end stage, wings,
  crossover, cyclorama, house steps, auditorium with raked seating.
- **Rigging** – house curtain, legs and borders, drops and tabs on battens that fly in and out
  with height readouts (or stay dead-hung where there is no fly tower).
- **Machinery** – revolve with angle control, orchestra pit lift and stage lift.
- **Lighting** – front-of-house, overhead bars and side positions with aimable lights, beam
  cones, colour and intensity, and a Work Light / Cue Lighting switch.
- **Set pieces** – a library of furniture and scenery, plus a Custom Piece Builder (boxes, flats,
  columns, ramps, stairs, hinged doors and lids, with exact sizes, position, rotation and material).
  Every library piece has its own settings (a door flat's door width, height, position, hinge
  side and swing; shelves, drawers, treads, gates...), and doors, lids, gates and drawers open
  and shut with one click or the O key. Any flat can be switched between plain, door, double
  doors, window and archway while keeping its size.
- **Hinged flats** – join flats together with hinges, edge to edge: book flats, folding
  screens, a door flat hinged to a wall. Fold them from in line to folded shut (towards the
  audience or back); the group moves together and the hinges are drawn on the flats.
- **Start-up screen** – the app's donkey picture (or your own logo or poster) with your theatre
  company's name, set in File › Start-up screen.
- **Easy controls** – Director, 3/4 Overview, Ground Plan, SL / SR Wing and Free Orbit cameras;
  click or outliner selection, drag, nudge, rotate in 15° or 1° steps, snap to grid, and
  warnings for pieces off the stage, in the sightlines, on the curtain line or too tall.
- **Scenes and export** – save several scenes or act states, save/open a project file (JSON) and
  export a PNG snapshot of the view.

## Download and install

Ready-made versions for every system are on the
[Releases page](https://github.com/Arrestis/StageCraft-3D/releases): nothing to compile.

| System | Versions |
| --- | --- |
| **Windows 10 / 11** | 64-bit (`x64`), 32-bit (`x86`) and ARM (`arm64`), each as an `.msi` installer, a portable `.zip` and a plain `.exe` |
| **Windows 7, 8, 8.1** | 64-bit and 32-bit (`windows7-x64`, `windows7-x86`): `.msi`, portable `.zip`, `.exe` |
| **macOS 11 or later** | One universal app for Apple Silicon and Intel Macs: `.dmg` and `.zip` |
| **Linux, PC and ARM** | For `x86_64` and `aarch64` (Raspberry Pi 4/5): AppImage and `.tar.gz` (any distribution), `.deb` (Debian, Ubuntu, Mint, Raspberry Pi OS), `.rpm` (Fedora, openSUSE, RHEL) and `.pkg.tar.zst` (Arch, Manjaro) |

Each release lists exactly which file to pick and how to install it. The files aren't
code-signed: Windows may say *More info › Run anyway*, and on a Mac right-click › **Open** the
first time.

### Making a new release

**Pushing with GitHub Desktop publishes the release.** `packaging\build-all.ps1` builds every
Windows and Linux file on a Windows PC and puts them in `releases/v<version>/`; commit and push
that folder and GitHub creates the release from those ready-made files, then builds and adds the
Mac version (a Mac app can only be built on a Mac). Step by step: [RELEASING.md](RELEASING.md).

The build needs Rust stable (targets `x86_64-pc-windows-msvc`, `i686-pc-windows-msvc`,
`aarch64-pc-windows-gnullvm`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`), Rust
nightly with `rust-src` (Windows 7 builds) and the WiX Toolset v5. Zig, llvm-mingw and the
AppImage runtime are downloaded automatically into `.tools\`.
## Building and running

You need [Rust](https://www.rust-lang.org/tools/install). Then, in this folder:

```
cargo run --release
```

`--screenshot picture.png` opens the window, draws about 120 frames, saves a picture and exits.

## Made with

Rust, [eframe / egui](https://github.com/emilk/egui) for the window and controls, and a small
built-in 3D renderer.

## License

Copyright © 2026 Doug Rumble.

Stagecraft 3D is free and open-source software: you can use it, share it and change it under
the terms of the [GNU General Public License, version 3](LICENSE) (`GPL-3.0-only`), as
published by the Free Software Foundation. It comes with no warranty. The app shows the same
notice, and the full licence, in File › About Stagecraft 3D.

The app icon is the project's own artwork. The folders in `vendor/` are two libraries (eframe,
MIT/Apache-2.0; rfd, MIT) with a small Windows 7 fix, described in each folder.
