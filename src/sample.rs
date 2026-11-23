//! A sample production (two scenes) and quick lighting looks.

use crate::library;
use crate::model::*;
use crate::venues;

pub fn empty_project(v: Venue) -> Project {
    let st = venues::fresh_state(&v);
    Project {
        app: "Stagecraft 3D".into(),
        version: 1,
        name: "Untitled production".into(),
        venue: v,
        scenes: vec![SceneState { name: "Act 1 Scene 1".into(), state: st }],
        current: 0,
        custom_library: Vec::new(),
        next_id: 1,
    }
}

fn find(lib: &[LibItem], name: &str) -> LibItem {
    lib.iter().find(|i| i.name == name).cloned().unwrap_or_else(|| lib[0].clone())
}

pub fn sample_project(v: Venue) -> Project {
    let lib = library::presets();
    let mut pr = empty_project(v.clone());
    pr.name = "The Lamplighter's Daughter".into();
    let w = v.pros_width;
    let d = v.stage_depth;
    let mut id = 1u32;
    let mut put = |pieces: &mut Vec<Piece>, name: &str, x: f32, z: f32, rot: f32| {
        let mut p = library::make_piece(&find(&lib, name), id, [x, 0.0, z]);
        p.rot = rot;
        id += 1;
        pieces.push(p);
    };

    // ---- Act 1 Scene 1: the parlour ----
    let mut s1 = venues::fresh_state(&v);
    let small = d < 5.5 || w < 7.5;
    if small {
        // Compact layout for intimate stages.
        let p = &mut s1.pieces;
        put(p, "Wide wall 3.6 x 3 m", 0.0, -d + 0.8, 0.0);
        put(p, "Flat 1.2 x 2.4 m", w * 0.38, -d + 1.75, -90.0);
        put(p, "Sofa (3-seat)", -0.2, -d + 1.35, 0.0);
        put(p, "Armchair", w * 0.2, -d + 2.1, -40.0);
        put(p, "Coffee table", -0.2, -d + 2.3, 0.0);
        put(p, "Standard lamp", -w * 0.3, -d + 1.2, 0.0);
        put(p, "Actor (scale figure 1.75 m)", -w * 0.25, -d + 2.2, 20.0);
        p[0].color = [168, 150, 120];
        let wall_w = (w * 0.62).min(3.6);
        let wall_h = p[0].size[1].min(v.pros_height - 0.2);
        library::set_param(&mut p[0], "width", wall_w);
        library::set_param(&mut p[0], "height", wall_h);
        p[1].name = "Side wall".into();
        p[1].color = [168, 150, 120];
    } else {
        let p = &mut s1.pieces;
        put(p, "Wide wall 3.6 x 3 m", 0.0, -d * 0.78, 0.0);
        put(p, "Door flat", w * 0.34, -d * 0.6, -70.0);
        put(p, "Window flat", -w * 0.34, -d * 0.58, 70.0);
        put(p, "Sofa (3-seat)", 0.0, -d * 0.45, 0.0);
        put(p, "Armchair", w * 0.2, -d * 0.36, -35.0);
        put(p, "Coffee table", 0.0, -d * 0.28, 0.0);
        put(p, "Bookshelf", -w * 0.18, -d * 0.74, 0.0);
        put(p, "Standard lamp", w * 0.17, -d * 0.7, 0.0);
        put(p, "Actor (scale figure 1.75 m)", -w * 0.12, -d * 0.24, 20.0);
        p[0].color = [168, 150, 120];
    }
    s1.notes = "Mrs Ashdown's parlour, late evening. Door SL leads to the hall; window SR looks onto the garden.".into();
    apply_look(&mut s1, "Warm interior");

    // ---- Act 1 Scene 2: the garden at night ----
    let mut s2 = venues::fresh_state(&v);
    // Keep the garden downstage of the night-sky drop (or the cyc).
    let back = s2.battens.iter().find(|b| b.kind == BattenKind::Drop).map(|b| b.dist).unwrap_or(d - 0.6);
    if small {
        let p = &mut s2.pieces;
        put(p, "Tree", -w * 0.28, -d + 1.6, 0.0);
        put(p, "Park bench", 0.4, -d + 1.5, 0.0);
        put(p, "Lamppost", w * 0.3, -d + 1.1, 0.0);
        put(p, "Actor (scale figure 1.75 m)", w * 0.05, -d + 2.3, -30.0);
    } else {
        let p = &mut s2.pieces;
        put(p, "Tree", -w * 0.3, -(back - 1.15), 0.0);
        put(p, "Picket fence", w * 0.12, -(back - 0.35), 0.0);
        put(p, "Lamppost", w * 0.28, -back * 0.62, 0.0);
        put(p, "Park bench", 0.2, -back * 0.5, 0.0);
        put(p, "Rock", -w * 0.2, -back * 0.3, 15.0);
        put(p, "Actor (scale figure 1.75 m)", 1.9, -back * 0.36, -30.0);
    }
    if small {
        // Low ceilings: keep tall scenery under the dead-hung pipes.
        let limit = (v.pros_height - 0.3).min(v.max_trim() - 0.5);
        for st in [&mut s1, &mut s2] {
            for p in st.pieces.iter_mut() {
                if p.size[1] > limit {
                    let k = limit / p.size[1];
                    p.size[1] = limit;
                    if p.category == "Exterior" {
                        p.size[0] *= k.max(0.75);
                        p.size[2] *= k.max(0.75);
                    }
                }
            }
        }
    }
    for b in s2.battens.iter_mut() {
        if b.kind == BattenKind::Drop {
            b.height = b.in_trim().min(v.max_trim());
            b.color = [40, 52, 96];
        }
    }
    if v.has_revolve {
        s2.revolve_angle = 90.0;
    }
    s2.notes = "The garden, an hour later. Fly in the night-sky drop; parlour trucks strike to the wings.".into();
    apply_look(&mut s2, "Moonlight");

    // Pieces standing on a revolve ride it (positions become relative to the disc).
    if v.has_revolve {
        for st in [&mut s1, &mut s2] {
            let ang = st.revolve_angle;
            for p in st.pieces.iter_mut() {
                let r = v.revolve_diameter * 0.5;
                let (hw, hd) = (p.size[0] * 0.5, p.size[2] * 0.5);
                let inside = [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)].iter().all(|&(cx, cz)| {
                    let q = crate::math::v3(cx, 0.0, cz).rot_y(p.rot);
                    let x = p.pos[0] + q.x;
                    let z = p.pos[2] + q.z - v.revolve_z;
                    (x * x + z * z).sqrt() <= r
                });
                if inside {
                    let l = crate::math::v3(p.pos[0], 0.0, p.pos[2] - v.revolve_z).rot_y(-ang);
                    p.pos[0] = l.x;
                    p.pos[2] = l.z;
                    p.rot -= ang;
                    p.on_revolve = true;
                }
            }
        }
    }
    pr.scenes = vec![
        SceneState { name: "Act 1 Scene 1 – The Parlour".into(), state: s1 },
        SceneState { name: "Act 1 Scene 2 – The Garden".into(), state: s2 },
    ];
    pr.next_id = id;
    pr
}

pub fn apply_look(s: &mut StageState, look: &str) {
    let warm = [1.0, 0.8, 0.58];
    let amber = [1.0, 0.62, 0.22];
    let white = [1.0, 0.97, 0.92];
    let cool = [0.64, 0.78, 1.0];
    let blue = [0.25, 0.38, 1.0];
    let lav = [0.72, 0.55, 1.0];
    let red = [1.0, 0.2, 0.12];
    let magenta = [1.0, 0.3, 0.65];
    for l in s.lights.iter_mut() {
        let foh = matches!(&l.mount, LightMount::Fixed { position_name, .. } if position_name.contains("FOH"));
        let boom = matches!(&l.mount, LightMount::Fixed { position_name, .. } if position_name.contains("boom"));
        let cyc = l.kind == LightKind::Flood;
        let (i, c) = match look {
            "Warm interior" => {
                if foh {
                    (0.75, warm)
                } else if boom {
                    (0.35, amber)
                } else if cyc {
                    (0.25, [1.0, 0.9, 0.6])
                } else {
                    (0.65, warm)
                }
            }
            "Bright day" => {
                if cyc {
                    (0.9, [0.55, 0.75, 1.0])
                } else {
                    (0.9, white)
                }
            }
            "Sunset" => {
                if foh {
                    (0.4, warm)
                } else if boom {
                    (0.7, red)
                } else if cyc {
                    (1.0, magenta)
                } else {
                    (0.6, amber)
                }
            }
            "Moonlight" => {
                if foh {
                    (0.15, cool)
                } else if boom {
                    (0.55, lav)
                } else if cyc {
                    (0.9, blue)
                } else {
                    (0.4, blue)
                }
            }
            "Dramatic side" => {
                if foh {
                    (0.0, white)
                } else if boom {
                    (1.0, white)
                } else if cyc {
                    (0.35, red)
                } else {
                    (0.08, cool)
                }
            }
            _ => (0.0, l.color),
        };
        l.intensity = i;
        l.color = c;
    }
    s.house_lights = if look == "Blackout" { 0.0 } else { 0.05 };
}
