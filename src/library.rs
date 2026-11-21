//! Preset library of furniture and set pieces, built from simple parts.

use crate::model::*;

fn part(shape: Shape, x: f32, y: f32, z: f32, w: f32, h: f32, d: f32) -> Part {
    Part { shape, offset: [x, y, z], size: [w, h, d], rot: 0.0, color: None, material: None }
}
fn bx(x: f32, y: f32, z: f32, w: f32, h: f32, d: f32) -> Part {
    part(Shape::Box, x, y, z, w, h, d)
}
fn cyl(x: f32, y: f32, z: f32, w: f32, h: f32, d: f32) -> Part {
    part(Shape::Cylinder, x, y, z, w, h, d)
}
trait Tint {
    fn c(self, rgb: [u8; 3]) -> Part;
    fn m(self, m: Material) -> Part;
    fn r(self, deg: f32) -> Part;
}
impl Tint for Part {
    fn c(mut self, rgb: [u8; 3]) -> Part {
        self.color = Some(rgb);
        self
    }
    fn m(mut self, m: Material) -> Part {
        self.material = Some(m);
        if self.color.is_none() {
            self.color = Some(m.default_color());
        }
        self
    }
    fn r(mut self, deg: f32) -> Part {
        self.rot = deg;
        self
    }
}

fn legs4(w: f32, d: f32, h: f32, t: f32, color: [u8; 3]) -> Vec<Part> {
    let x = w * 0.5 - t * 0.5;
    let z = d * 0.5 - t * 0.5;
    vec![
        bx(-x, 0.0, -z, t, h, t).c(color),
        bx(x, 0.0, -z, t, h, t).c(color),
        bx(-x, 0.0, z, t, h, t).c(color),
        bx(x, 0.0, z, t, h, t).c(color),
    ]
}

fn item(name: &str, cat: &str, mat: Material, color: [u8; 3], parts: Vec<Part>) -> LibItem {
    LibItem { name: name.into(), category: cat.into(), material: mat, color, parts }
}

pub const CATEGORIES: [&str; 5] = ["Furniture", "Walls & Doors", "Levels & Stairs", "Exterior", "Rehearsal"];

pub fn presets() -> Vec<LibItem> {
    let dark_wood = [92, 60, 38];
    let cushion = [168, 92, 92];
    let mut v = Vec::new();

    // ---------- Furniture ----------
    let mut sofa = vec![
        bx(0.0, 0.12, 0.05, 1.7, 0.3, 0.75),
        bx(0.0, 0.12, -0.33, 2.0, 0.7, 0.2),
        bx(-0.9, 0.12, 0.05, 0.2, 0.5, 0.85),
        bx(0.9, 0.12, 0.05, 0.2, 0.5, 0.85),
        bx(-0.42, 0.42, 0.08, 0.8, 0.12, 0.66).c(cushion),
        bx(0.42, 0.42, 0.08, 0.8, 0.12, 0.66).c(cushion),
    ];
    sofa.extend(legs4(1.9, 0.75, 0.12, 0.06, dark_wood));
    v.push(item("Sofa (3-seat)", "Furniture", Material::Fabric, [110, 52, 62], sofa));

    let mut chair = vec![
        bx(0.0, 0.12, 0.05, 0.6, 0.3, 0.7),
        bx(0.0, 0.12, -0.32, 0.9, 0.85, 0.18),
        bx(-0.37, 0.12, 0.05, 0.16, 0.5, 0.8),
        bx(0.37, 0.12, 0.05, 0.16, 0.5, 0.8),
        bx(0.0, 0.42, 0.08, 0.58, 0.12, 0.62).c(cushion),
    ];
    chair.extend(legs4(0.85, 0.75, 0.12, 0.06, dark_wood));
    v.push(item("Armchair", "Furniture", Material::Fabric, [70, 90, 110], chair));

    let mut ct = vec![bx(0.0, 0.38, 0.0, 1.1, 0.05, 0.6), bx(0.0, 0.12, 0.0, 1.0, 0.03, 0.5)];
    ct.extend(legs4(1.05, 0.55, 0.38, 0.05, dark_wood));
    v.push(item("Coffee table", "Furniture", Material::Timber, [130, 88, 54], ct));

    let mut dt = vec![bx(0.0, 0.71, 0.0, 1.8, 0.05, 0.9)];
    dt.extend(legs4(1.7, 0.8, 0.71, 0.07, dark_wood));
    v.push(item("Dining table", "Furniture", Material::Timber, [130, 88, 54], dt));

    let mut dc = vec![bx(0.0, 0.43, 0.0, 0.44, 0.05, 0.44), bx(0.0, 0.48, -0.2, 0.44, 0.45, 0.04)];
    dc.extend(legs4(0.42, 0.42, 0.43, 0.04, dark_wood));
    v.push(item("Dining chair", "Furniture", Material::Timber, [120, 80, 50], dc));

    let mut desk = vec![
        bx(0.0, 0.72, 0.0, 1.4, 0.04, 0.7),
        bx(0.45, 0.0, 0.0, 0.45, 0.72, 0.66),
        bx(0.45, 0.5, 0.335, 0.4, 0.15, 0.01).c([170, 140, 90]),
        bx(0.45, 0.3, 0.335, 0.4, 0.15, 0.01).c([170, 140, 90]),
        bx(-0.65, 0.0, 0.0, 0.06, 0.72, 0.66),
    ];
    desk.push(bx(-0.2, 0.76, -0.1, 0.35, 0.25, 0.04).c([30, 30, 34]));
    v.push(item("Writing desk", "Furniture", Material::Timber, [110, 72, 44], desk));

    let bed = vec![
        bx(0.0, 0.0, 0.0, 1.5, 0.3, 2.0).c(dark_wood),
        bx(0.0, 0.3, 0.05, 1.45, 0.22, 1.9).c([236, 232, 222]),
        bx(0.0, 0.52, 0.25, 1.47, 0.04, 1.5).c([90, 110, 150]),
        bx(-0.35, 0.52, -0.75, 0.55, 0.12, 0.35).c([245, 245, 240]),
        bx(0.35, 0.52, -0.75, 0.55, 0.12, 0.35).c([245, 245, 240]),
        bx(0.0, 0.0, -0.98, 1.55, 1.1, 0.08).c(dark_wood),
    ];
    v.push(item("Double bed", "Furniture", Material::Timber, [92, 60, 38], bed));

    let mut shelf = vec![
        bx(-0.48, 0.0, 0.0, 0.04, 2.0, 0.35),
        bx(0.48, 0.0, 0.0, 0.04, 2.0, 0.35),
        bx(0.0, 0.0, -0.165, 1.0, 2.0, 0.02),
    ];
    let book_cols = [[150, 40, 40], [40, 80, 130], [60, 110, 60], [170, 140, 60], [90, 50, 110]];
    for i in 0..5 {
        let y = 0.02 + i as f32 * 0.48;
        shelf.push(bx(0.0, y, 0.0, 0.92, 0.03, 0.33));
        if i < 4 {
            for j in 0..6 {
                let x = -0.36 + j as f32 * 0.145;
                let h = 0.26 + ((i * 7 + j * 3) % 5) as f32 * 0.025;
                shelf.push(bx(x, y + 0.03, 0.01, 0.11, h, 0.24).c(book_cols[(i + j) % 5]));
            }
        }
    }
    v.push(item("Bookshelf", "Furniture", Material::Timber, [100, 66, 40], shelf));

    let lamp = vec![
        cyl(0.0, 0.0, 0.0, 0.3, 0.03, 0.3).c([50, 50, 50]),
        cyl(0.0, 0.03, 0.0, 0.03, 1.35, 0.03).c([60, 60, 60]),
        part(Shape::Cone, 0.0, 1.3, 0.0, 0.45, 0.35, 0.45).c([235, 220, 180]),
    ];
    v.push(item("Standard lamp", "Furniture", Material::Metal, [60, 60, 60], lamp));

    let piano = vec![
        bx(0.0, 0.0, 0.0, 1.5, 1.25, 0.45),
        bx(0.0, 0.65, 0.35, 1.45, 0.08, 0.3),
        bx(0.0, 0.73, 0.36, 1.3, 0.02, 0.22).c([240, 240, 235]),
        bx(-0.68, 0.0, 0.35, 0.08, 0.65, 0.08),
        bx(0.68, 0.0, 0.35, 0.08, 0.65, 0.08),
    ];
    v.push(item("Upright piano", "Furniture", Material::Timber, [40, 26, 20], piano));

    // ---------- Walls & doors ----------
    let flat_col = [190, 180, 150];
    v.push(item("Flat 1.2 x 2.4 m", "Walls & Doors", Material::PaintedWood, flat_col, vec![bx(0.0, 0.0, 0.0, 1.2, 2.4, 0.06)]));
    v.push(item("Wide wall 3.6 x 3 m", "Walls & Doors", Material::PaintedWood, flat_col, vec![bx(0.0, 0.0, 0.0, 3.6, 3.0, 0.08)]));
    v.push(item(
        "Door flat",
        "Walls & Doors",
        Material::PaintedWood,
        flat_col,
        vec![
            bx(-0.8, 0.0, 0.0, 0.6, 2.7, 0.08),
            bx(0.8, 0.0, 0.0, 0.6, 2.7, 0.08),
            bx(0.0, 2.1, 0.0, 1.0, 0.6, 0.08),
            bx(0.0, 0.0, -0.06, 0.9, 2.1, 0.04).c([120, 70, 40]).r(-35.0),
            bx(-0.52, 0.0, 0.0, 0.06, 2.16, 0.12).c([235, 230, 220]),
            bx(0.52, 0.0, 0.0, 0.06, 2.16, 0.12).c([235, 230, 220]),
            bx(0.0, 2.1, 0.0, 1.1, 0.08, 0.12).c([235, 230, 220]),
        ],
    ));
    v.push(item(
        "Window flat",
        "Walls & Doors",
        Material::PaintedWood,
        flat_col,
        vec![
            bx(0.0, 0.0, 0.0, 2.0, 0.9, 0.08),
            bx(0.0, 2.1, 0.0, 2.0, 0.6, 0.08),
            bx(-0.75, 0.9, 0.0, 0.5, 1.2, 0.08),
            bx(0.75, 0.9, 0.0, 0.5, 1.2, 0.08),
            bx(0.0, 0.9, 0.0, 1.0, 1.2, 0.02).m(Material::Glass),
            bx(0.0, 0.9, 0.02, 0.05, 1.2, 0.04).c([235, 230, 220]),
            bx(0.0, 1.48, 0.02, 1.0, 0.05, 0.04).c([235, 230, 220]),
            bx(0.0, 0.86, 0.06, 1.1, 0.05, 0.16).c([235, 230, 220]),
        ],
    ));
    v.push(item(
        "Archway",
        "Walls & Doors",
        Material::Stone,
        [200, 190, 170],
        vec![
            bx(-1.0, 0.0, 0.0, 0.5, 2.2, 0.3),
            bx(1.0, 0.0, 0.0, 0.5, 2.2, 0.3),
            part(Shape::ArchTop, 0.0, 2.2, 0.0, 1.5, 0.75, 0.3),
            bx(0.0, 2.95, 0.0, 2.5, 0.35, 0.3),
            bx(-1.0, 2.15, 0.0, 0.6, 0.1, 0.36),
            bx(1.0, 2.15, 0.0, 0.6, 0.1, 0.36),
        ],
    ));
    v.push(item(
        "Column",
        "Walls & Doors",
        Material::Stone,
        [215, 208, 195],
        vec![
            bx(0.0, 0.0, 0.0, 0.55, 0.2, 0.55),
            cyl(0.0, 0.2, 0.0, 0.4, 2.6, 0.4),
            bx(0.0, 2.8, 0.0, 0.55, 0.2, 0.55),
        ],
    ));
    v.push(item(
        "Angled wall unit",
        "Walls & Doors",
        Material::PaintedWood,
        flat_col,
        vec![bx(-0.85, 0.0, 0.0, 1.8, 2.7, 0.08).r(25.0), bx(0.85, 0.0, 0.0, 1.8, 2.7, 0.08).r(-25.0)],
    ));

    // ---------- Levels & stairs ----------
    v.push(item("Rostrum 2.4 x 1.2 x 0.4 m", "Levels & Stairs", Material::PaintedWood, [60, 60, 66], vec![bx(0.0, 0.0, 0.0, 2.4, 0.4, 1.2)]));
    v.push(item("Platform 2.4 x 2.4 x 0.9 m", "Levels & Stairs", Material::PaintedWood, [70, 70, 76], vec![bx(0.0, 0.0, 0.0, 2.4, 0.9, 2.4)]));
    v.push(item("Stair unit (4 treads)", "Levels & Stairs", Material::PaintedWood, [80, 76, 72], vec![part(Shape::Stairs(4), 0.0, 0.0, 0.0, 1.0, 0.8, 1.2)]));
    v.push(item(
        "Split-level riser + stairs",
        "Levels & Stairs",
        Material::PaintedWood,
        [70, 70, 76],
        vec![bx(0.0, 0.0, -0.6, 2.4, 0.6, 1.2), part(Shape::Stairs(3), 0.0, 0.0, 0.45, 1.0, 0.6, 0.9)],
    ));
    v.push(item(
        "Escape stairs (wing)",
        "Levels & Stairs",
        Material::Metal,
        [70, 72, 78],
        vec![
            part(Shape::Stairs(8), 0.0, 0.0, 0.0, 0.9, 1.8, 2.0),
            bx(0.0, 0.0, -1.5, 0.9, 1.8, 1.0),
            bx(-0.45, 1.8, -0.5, 0.03, 0.9, 3.0).c([200, 200, 60]),
        ],
    ));
    v.push(item("Ramp 1.2 x 2.4 m", "Levels & Stairs", Material::PaintedWood, [80, 76, 72], vec![part(Shape::Ramp, 0.0, 0.0, 0.0, 1.2, 0.4, 2.4)]));

    // ---------- Exterior ----------
    v.push(item(
        "Tree",
        "Exterior",
        Material::Foliage,
        [70, 120, 60],
        vec![
            cyl(0.0, 0.0, 0.0, 0.3, 2.2, 0.3).c([90, 64, 42]),
            part(Shape::Sphere, 0.0, 1.6, 0.0, 1.9, 1.6, 1.9),
            part(Shape::Sphere, 0.35, 2.5, 0.1, 1.3, 1.2, 1.3).c([84, 136, 70]),
            part(Shape::Sphere, -0.4, 2.3, -0.2, 1.1, 1.0, 1.1).c([60, 108, 52]),
        ],
    ));
    v.push(item(
        "Rock",
        "Exterior",
        Material::Stone,
        [130, 124, 116],
        vec![part(Shape::Sphere, 0.0, -0.1, 0.0, 1.4, 0.9, 1.0), part(Shape::Sphere, 0.45, -0.1, 0.25, 0.8, 0.6, 0.7)],
    ));
    v.push(item(
        "Lamppost",
        "Exterior",
        Material::Metal,
        [40, 44, 40],
        vec![
            cyl(0.0, 0.0, 0.0, 0.3, 0.3, 0.3),
            cyl(0.0, 0.3, 0.0, 0.1, 3.0, 0.1),
            bx(0.0, 3.3, 0.0, 0.32, 0.4, 0.32).m(Material::Glass).c([255, 230, 160]),
            part(Shape::Cone, 0.0, 3.7, 0.0, 0.45, 0.25, 0.45),
        ],
    ));
    let mut bench = vec![
        bx(0.0, 0.42, 0.0, 1.6, 0.05, 0.42),
        bx(0.0, 0.55, -0.2, 1.6, 0.35, 0.04),
    ];
    bench.extend(legs4(1.5, 0.4, 0.42, 0.06, [40, 40, 40]));
    v.push(item("Park bench", "Exterior", Material::Timber, [130, 90, 50], bench));
    let mut fence = Vec::new();
    for i in 0..9 {
        fence.push(bx(-1.0 + i as f32 * 0.25, 0.0, 0.0, 0.09, 1.0, 0.03));
    }
    fence.push(bx(0.0, 0.25, -0.03, 2.2, 0.08, 0.03));
    fence.push(bx(0.0, 0.75, -0.03, 2.2, 0.08, 0.03));
    v.push(item("Picket fence", "Exterior", Material::PaintedWood, [235, 235, 228], fence));

    // ---------- Rehearsal ----------
    v.push(item("Rehearsal block", "Rehearsal", Material::Masking, [36, 36, 40], vec![bx(0.0, 0.0, 0.0, 0.45, 0.45, 0.45)]));
    v.push(item("Long rehearsal block", "Rehearsal", Material::Masking, [36, 36, 40], vec![bx(0.0, 0.0, 0.0, 0.9, 0.45, 0.45)]));
    v.push(item(
        "Actor (scale figure 1.75 m)",
        "Rehearsal",
        Material::Fabric,
        [70, 110, 160],
        vec![
            bx(-0.1, 0.0, 0.0, 0.14, 0.85, 0.18).c([50, 50, 70]),
            bx(0.1, 0.0, 0.0, 0.14, 0.85, 0.18).c([50, 50, 70]),
            bx(0.0, 0.85, 0.0, 0.42, 0.6, 0.24),
            bx(-0.27, 0.8, 0.0, 0.1, 0.62, 0.12),
            bx(0.27, 0.8, 0.0, 0.1, 0.62, 0.12),
            part(Shape::Sphere, 0.0, 1.5, 0.0, 0.22, 0.25, 0.24).c([224, 186, 150]),
        ],
    ));
    v.push(item(
        "Spike mark (floor tape)",
        "Rehearsal",
        Material::PaintedWood,
        [240, 200, 40],
        vec![bx(0.0, 0.0, 0.0, 0.3, 0.005, 0.05), bx(0.0, 0.0, 0.0, 0.05, 0.005, 0.3)],
    ));
    v
}

pub fn make_piece(item: &LibItem, id: u32, pos: [f32; 3]) -> Piece {
    let (lo, hi) = item.bounds();
    let size = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    // Re-centre the parts so the piece origin is the centre of its footprint, base at 0.
    let cx = (lo[0] + hi[0]) * 0.5;
    let cz = (lo[2] + hi[2]) * 0.5;
    let parts = item
        .parts
        .iter()
        .map(|p| {
            let mut q = p.clone();
            q.offset[0] -= cx;
            q.offset[1] -= lo[1];
            q.offset[2] -= cz;
            q
        })
        .collect();
    Piece {
        id,
        name: item.name.clone(),
        category: item.category.clone(),
        parts,
        base_size: size,
        size,
        pos,
        rot: 0.0,
        material: item.material,
        color: item.color,
        visible: true,
        locked: false,
        on_revolve: false,
        flown_on: None,
    }
}
