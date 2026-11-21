# Changelog

## v0.4.1 – venues from their own documents

- **TAS Hoskins Centre** rebuilt from its technical specifications: 12 m x 7.65 m proscenium on a
  12.2 m x 7.35 m stage, four lighting trusses (two motorised), movable masking tabs, a 7.5 m x
  2.5 m pit, 230 seats with East and West side galleries, and its FOH gallery, perches and booms.
- **Capitol Theatre Tamworth** now hangs its real 17-line hanging plot: gold house curtain, false
  proscenium, legs and borders at 4.7 m x 10 m, four LX bars, two sets of black tabs and the white
  cyc; 402 seats with the centre cross aisle; FOH bridge, perches and box booms on the side walls.
- **Genesian Theatre**: Rozelle gets its teal-and-gold room, light timber proscenium and steep
  seating with carpeted aisle stairs both sides; the old Kent Street church gets oak panelling,
  a timber roof, red velvet seats and its rear gallery.
- **Lazenby Hall** gets its timber hall look, side gallery and screens beside the stage; the
  benchmark house gets red and gilt with a balcony.
- New Venue settings: room look, side galleries, rear gallery, aisle stairs on both sides, cross
  aisle and screens. The Director view now sits far enough back to see the whole opening.

## v0.4.0 – new home: StageCraft-3D

- New app icon (the project's donkey-head artwork) on every system.
- Windows: 64-bit, 32-bit and **ARM64** for Windows 10/11, plus **Windows 7, 8 and 8.1** (64- and
  32-bit, drawing with OpenGL) – each as installer, portable zip and plain exe. The C runtime is
  built in, so no Visual C++ redistributable is needed.
- Linux: PC (x86_64) and **ARM (aarch64)** builds, each as AppImage, tar.gz, .deb, .rpm and a new
  **Arch Linux** package.
- macOS: universal app (.dmg, .zip) built by GitHub for every release.
- Licensed under the GNU GPL v3.
- `STAGECRAFT_OPENGL=1` forces the OpenGL renderer for older graphics drivers.

## v0.3.1

- Complete release: Windows (64- and 32-bit installer, portable zip and .exe), Linux (AppImage,
  .tar.gz, .deb, .rpm) and macOS (.dmg, .zip) for the same version.
- Linux versions are now built ready-to-run on Windows; only the Mac version is built by GitHub.
- New "Add files to latest release" shortcut, a step-by-step release guide (RELEASING.md),
  bug-report and feature-request forms, and an automatic build check.
- Stray copies of git's own system files (committed by mistake in "updated files") are now
  ignored, so they can't be committed again.

## v0.3.0 – first public release

### Venues
- Stage presets for Armidale Playhouse, TAS Hoskins Centre, Lazenby Hall (UNE), Capitol Theatre
  Tamworth, Genesian Theatre (Rozelle and its former Kent Street church), a large proscenium
  house, a flexible black box and a Custom Stage – all with editable dimensions.
- Armidale Playhouse rebuilt from the venue's as-built floor plan and photos: 8.3 m x 3.75 m open
  end stage on a 450 mm riser, 3.2 m ceiling sloping to 2.3 m on stage left, sound lock and green
  room behind, about 120 raked seats with a house-right aisle, control box and the real lighting
  positions. The app opens on this venue.
- Capitol Theatre Tamworth follows its published technical specification.

### Stage and set
- 3D stage with proscenium or open end stage, wings, crossover, cyclorama, steps, raked seating.
- Fly bars, curtains, legs, borders and drops with height readouts; revolve, pit lift, stage lift.
- Lighting rig with aimable lights, beam cones, colour and intensity; Work Light / Cue Lighting.
- Furniture and scenery library plus a Custom Piece Builder.
- Warnings for pieces off the stage, outside sightlines, on curtain lines or too tall for the ceiling.
- Scenes, project save/open (JSON) and PNG snapshots.

### Downloads
- Windows 64-bit and 32-bit: installer (.msi), portable zip and standalone .exe.
- Linux x86_64: AppImage, portable .tar.gz, .deb and .rpm.
- macOS universal (Apple Silicon and Intel): .dmg and .zip, built by GitHub for each release.
