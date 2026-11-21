//! Venue presets drawn from research, plus default rigging and lighting for a venue.

use crate::model::*;

fn base(key: &str, name: &str) -> Venue {
    Venue {
        key: key.into(),
        name: name.into(),
        location: String::new(),
        notes: String::new(),
        kind: VenueKind::Proscenium,
        capacity: 200,
        pros_width: 9.0,
        pros_height: 5.0,
        stage_depth: 8.0,
        wing_width: 3.0,
        apron_depth: 1.0,
        grid_height: 0.0,
        pipe_height: 5.5,
        fly_lines: 8,
        fly_system: String::new(),
        rake_deg: 0.0,
        stage_height: 0.9,
        has_pit: false,
        pit_width: 8.0,
        pit_depth: 2.5,
        pit_is_lift: false,
        has_revolve: false,
        revolve_diameter: 6.0,
        revolve_z: -4.0,
        has_stage_lift: false,
        lift_width: 2.4,
        lift_depth: 1.2,
        lift_x: 0.0,
        lift_z: -6.0,
        has_crossover: true,
        has_house_steps: true,
        curtain_style: CurtainStyle::Traveller,
        house_width: 14.0,
        house_rise_per_row: 0.12,
        fly_gallery_height: 0.0,
        confidence: String::new(),
        open_end_stage: false,
        sl_soffit_width: 0.0,
        sl_soffit_height: 0.0,
        dark_room: false,
        seats_per_row_fixed: 0,
        side_aisle: false,
        control_box: false,
        tech_notes: String::new(),
        look: RoomLook::Classic,
        side_galleries: false,
        rear_gallery: false,
        aisles_both_sides: false,
        cross_aisle_after: 0,
        foh_dist: 0.0,
        perch_dist: 0.0,
        foh_boom_dist: 0.0,
        side_screens: false,
    }
}

/// The preset a (possibly edited) venue came from.
pub fn base_key(v: &Venue) -> &str {
    v.key.strip_suffix("_edited").unwrap_or(&v.key)
}

pub fn presets() -> Vec<Venue> {
    let mut v = Vec::new();

    // Armidale Playhouse – compact community theatre.
    let mut p = base("armidale_playhouse", "Armidale Playhouse");
    p.location = "Armidale, NSW".into();
    p.location = "309 Beardy Street, Armidale, NSW".into();
    p.notes = "Intimate black-box style end stage with no proscenium arch. The performance space is 8.3 m wide \
x 3.75 m deep on a 450 mm high stage riser, with a 3.2 m ceiling over the main 5.9 m and a lower ceiling \
sloping down to 2.3 m on the stage-left side. Black drapes mask about 1.2 m each side, black traverse \
curtains hang along the charcoal back wall, and a sound lock behind it leads to the green room and \
dressing room. Steps with handrails come down from the stage-left front corner. About 120 red seats in \
10 raked rows with the aisle stairs on house right; control box at the back. No flying: everything hangs \
from fixed bars, so scenery must be carried on and off."
        .into();
    p.tech_notes = "Lighting: front bar over the stage edge and a second front-of-house bar over the seats \
(profiles at the ends, LED PARs and fresnels in the middle), a short bar inside the stage ceiling, side \
positions behind the masking and two LED PARs on the house-left wall. Control: compact LED/dimmer \
lighting desk with touch screen, 20-input digital sound desk in the control box, PA mixer-amp with \
foyer / auditorium / green-room zones, hearing-loop amplifier, and a patch panel with dimmer, pit, \
green-room and comms lines."
        .into();
    p.capacity = 120;
    p.pros_width = 5.9;
    p.pros_height = 3.2;
    p.stage_depth = 3.75;
    p.wing_width = 1.2;
    p.apron_depth = 0.0;
    p.grid_height = 0.0;
    p.pipe_height = 3.05;
    p.fly_lines = 4;
    p.fly_system = "Fixed (dead-hung) bars – no flying".into();
    p.stage_height = 0.45;
    p.has_crossover = true;
    p.house_width = 8.3;
    p.house_rise_per_row = 0.22;
    p.open_end_stage = true;
    p.sl_soffit_width = 2.4;
    p.sl_soffit_height = 2.3;
    p.dark_room = true;
    p.look = RoomLook::Studio;
    p.seats_per_row_fixed = 12;
    p.side_aisle = true;
    p.control_box = true;
    p.confidence = "Stage 8.3 m x 3.75 m, 450 mm riser, 3.2 m / 2.3 m ceilings and the room layout from the \
as-built floor plan (2017/2020) and venue photos; seat rake and lighting bar heights are estimated from photos."
        .into();
    v.push(p);

    // TAS Hoskins Centre.
    let mut p = base("tas_hoskins", "TAS Hoskins Centre");
    p.location = "The Armidale School, Armidale, NSW".into();
    p.notes = "The school's Michael Hoskins Creative Arts Centre: a 230-seat proscenium theatre with an unraked, \
unmarked stage 12.2 m wide x 7.35 m deep, a 12 m proscenium opening 7.65 m high and limited flying. \
Instead of a fly tower it has four lighting trusses: one on the proscenium and one on the rear wall \
(fixed chain blocks), and two on motorised trolleys and winches that lower for bump-in (only two may be \
down at once, and none may move during a show). Masking is two movable tabs either side of the stage. \
The orchestra pit (7.5 m x 2.5 m, for 16 players) is set at 600 mm or 1200 mm deep with relocatable \
desking; with the pit floored, rows A and B add 21 seats (251). Main tiered seating (rows C-L, 186 seats) \
plus an East and a West side gallery of 20 seats each. Lighting, sound and AV run from an open bio box \
at the back."
        .into();
    p.tech_notes = "Lighting positions (from the setting line): orchestra bar 1.0 m over the pit, FOH perches \
1.3 m above the front galleries, FOH gallery 6.5 m (two parallel bars, 11 m usable), audience booms 6.7 m, \
TD bar 8 m and FOH booms 11 m in front of the galleries; four side LX bars on request. Control: ETC Ion-type \
desk with fader wing, 2 DMX universes. Stock: about 30 LED PARs and hybrids, 40+ profiles, PCs, 4 cyc \
floods, 2 follow spots, PAR 64s; hazer, fog and low fog. Sound: digital desk in the bio box, L/C/R mains with \
subs and rear fills. AV: two projectors and a motorised 200-inch 4:3 screen on the proscenium truss. \
Staging: ten 1.2 x 2.4 m steel rostra at chosen heights."
        .into();
    p.capacity = 230;
    p.pros_width = 12.0;
    p.pros_height = 7.65;
    p.stage_depth = 7.35;
    p.wing_width = 0.1;
    p.apron_depth = 1.0;
    p.grid_height = 0.0;
    p.pipe_height = 8.4;
    p.fly_lines = 4;
    p.fly_system = "4 lighting trusses: 2 motorised (trolleys & winches), 2 static chain blocks".into();
    p.stage_height = 0.9;
    p.has_pit = true;
    p.pit_width = 7.5;
    p.pit_depth = 2.5;
    p.pit_is_lift = true;
    p.house_width = 16.0;
    p.house_rise_per_row = 0.22;
    p.seats_per_row_fixed = 21;
    p.side_galleries = true;
    p.control_box = true;
    p.foh_dist = 6.5;
    p.perch_dist = 1.3;
    p.foh_boom_dist = 11.0;
    p.confidence = "Seating, stage 12.2 m x 7.35 m, 12 m x 7.65 m proscenium, pit, trusses, masking and lighting \
positions from the venue's technical specifications (01/2023). Stage height, house rake, truss trim heights and \
the position of the second motorised truss are estimates; the room's colours are not documented."
        .into();
    v.push(p);

    // Lazenby Hall, UNE.
    let mut p = base("lazenby_hall", "Lazenby Hall (UNE)");
    p.location = "University of New England, Armidale, NSW".into();
    p.notes = "The region's large hall: a wide stage opening framed in black drapes, facing a semi-raked \
auditorium of about 856 seats, with winched fly lines. Timber-lined walls and a slatted timber ceiling, \
lighting bars over the front of the house, a side gallery and projection screens either side of the stage \
(used for graduations and concerts as well as shows). The wide house means extreme side seats see deep \
into the wings, so check sightlines."
        .into();
    p.look = RoomLook::TimberHall;
    p.side_galleries = true;
    p.side_screens = true;
    p.capacity = 856;
    p.pros_width = 12.0;
    p.pros_height = 6.0;
    p.stage_depth = 10.0;
    p.wing_width = 4.0;
    p.apron_depth = 1.5;
    p.grid_height = 14.0;
    p.fly_lines = 18;
    p.fly_system = "Winched fly lines".into();
    p.stage_height = 1.0;
    p.has_pit = true;
    p.pit_width = 10.0;
    p.pit_depth = 2.5;
    p.pit_is_lift = false;
    p.house_width = 24.0;
    p.house_rise_per_row = 0.08;
    p.house_rise_per_row = 0.06;
    p.confidence = "856 seats, proscenium and winched flying from the brief; the room's finishes, gallery and \
screens from a photo of the hall. Dimensions and pit are estimates."
        .into();
    v.push(p);

    // Capitol Theatre Tamworth – published technical specifications.
    let mut p = base("capitol_tamworth", "Capitol Theatre Tamworth");
    p.location = "Centrepoint, Peel Street, Tamworth, NSW".into();
    p.notes = "Purpose-built 402-seat proscenium arch theatre with a single-tier raked auditorium split by a \
centre cross aisle (rows A-E in front, F-S behind), an orchestra pit lift and 17 lines of mixed motor/manual \
winch flying. Published specs: proscenium 10.6 m wide x 5.0 m high, 9.4 m from setting line to rear wall, \
8 m from centre line to each side wall, 11.7 m to the grid beams, unraked stage, 1.475 m apron, 8 m x 3 m \
pit lift (any height) with the stage 1.0 m above the auditorium, crossover behind the upstage fly line, \
fly gallery on the prompt side at 7 m. The hanging plot below follows the venue's: gold house curtain \
(French action), false proscenium at 5 m x 10.6 m, legs and borders at 4.7 m x 10 m, four LX bars at \
5.7-5.8 m, two sets of black tabs and a white cyclorama. No build under the smoke curtain line."
        .into();
    p.tech_notes = "Lighting positions: FOH bridge 14 m from the setting line (13 m usable), rear wall bar 26 m, \
perches 2.5 m and box booms 9.5 m on the side walls, four motorised LX bars on stage (not swingable) and a \
truss at 6 m. Standard rig: profiles, fresnels, LED PARs and moving washes/spots, Aurora cyc floods on LX 4. \
Control: ETC Ion-type desk, 2 DMX universes, dimmer room on level 2 by the prompt-side fly gallery. Sound: \
digital desk at the rear centre of the auditorium or in the bio box, flown L/C/R mains with delays and subs. \
AV: 6000-lumen projector and a 5.5 x 3.1 m screen that flies. Six 2.4 x 1.2 m risers at 300 or 600 mm. \
Load-in through the sub-stage dock and up on the pit lift (doors 2.4 m x 2.0 m)."
        .into();
    p.capacity = 402;
    p.seats_per_row_fixed = 22;
    p.cross_aisle_after = 5;
    p.control_box = true;
    p.foh_dist = 14.0;
    p.perch_dist = 2.5;
    p.foh_boom_dist = 9.5;
    p.pros_width = 10.6;
    p.pros_height = 5.0;
    p.stage_depth = 9.4;
    p.wing_width = 2.7;
    p.apron_depth = 1.475;
    p.grid_height = 11.7;
    p.fly_lines = 17;
    p.fly_system = "Motor winches (LX & scenery) + manual winches (legs & borders)".into();
    p.stage_height = 1.0;
    p.has_pit = true;
    p.pit_width = 8.0;
    p.pit_depth = 3.0;
    p.pit_is_lift = true;
    p.house_width = 17.0;
    p.house_rise_per_row = 0.24;
    p.fly_gallery_height = 7.0;
    p.confidence = "Stage, pit, flying, hanging plot, drapery and lighting positions from the venue's technical \
specification (22/08/2025), section and seating plan. House rake and the room's colours are estimates."
        .into();
    v.push(p);

    // Genesian Theatre – Rozelle.
    let mut p = base("genesian_rozelle", "Genesian Theatre (Rozelle)");
    p.location = "2B Gordon Street, Rozelle, Sydney, NSW".into();
    p.notes = "Sydney's long-running amateur company, now in the St Joseph's Church precinct in Rozelle after \
leaving its historic Kent Street church home. A compact purpose-fitted theatre: steeply raked red seats in a \
single block with red-carpeted aisle stairs and handrails on both sides, teal walls with gold trim, and a \
light timber proscenium frame round a black stage. Shallow stage, minimal wing space, no fly tower – \
scenery is built on the floor and changed by hand."
        .into();
    p.look = RoomLook::TealGold;
    p.aisles_both_sides = true;
    p.seats_per_row_fixed = 10;
    p.control_box = true;
    p.capacity = 110;
    p.pros_width = 7.0;
    p.pros_height = 3.6;
    p.stage_depth = 5.0;
    p.wing_width = 1.3;
    p.apron_depth = 0.6;
    p.grid_height = 0.0;
    p.pipe_height = 4.2;
    p.fly_lines = 6;
    p.fly_system = "Dead-hung pipes (estimated)".into();
    p.stage_height = 0.5;
    p.house_width = 8.6;
    p.house_rise_per_row = 0.3;
    p.confidence = "Address and move confirmed; the room's layout and finishes from photos of the new theatre. \
Stage and seating figures are estimates."
        .into();
    v.push(p);

    // Genesian – former Kent Street church (historic reference).
    let mut p = base("genesian_kent_st", "Genesian (former Kent St church)");
    p.location = "420 Kent Street, Sydney CBD (historic)".into();
    p.notes = "The company's former home: an 1868 church converted into a tiny theatre. Narrow nave with a \
timber hammer-beam roof, cream walls over dark oak panelling, red velvet seats and a gallery at the back. \
The stage sits inside a panelled oak proscenium with a lighting bar hung just in front of it and steps \
down to the house on both sides; almost no wing space and no flying – a useful 'worst case' to test \
touring sets."
        .into();
    p.look = RoomLook::TimberChurch;
    p.rear_gallery = true;
    p.foh_dist = 1.2;
    p.capacity = 100;
    p.pros_width = 6.0;
    p.pros_height = 3.4;
    p.stage_depth = 4.2;
    p.wing_width = 0.9;
    p.apron_depth = 0.5;
    p.grid_height = 0.0;
    p.pipe_height = 4.0;
    p.fly_lines = 5;
    p.fly_system = "Dead-hung pipes".into();
    p.stage_height = 0.6;
    p.has_crossover = false;
    p.house_width = 9.0;
    p.house_rise_per_row = 0.1;
    p.confidence = "History confirmed; the room's finishes, gallery and lighting bar from photos. Dimensions are \
estimates."
        .into();
    v.push(p);

    // Benchmark: large West End / Broadway style house.
    let mut p = base("grand_proscenium", "Grand Proscenium House (benchmark)");
    p.location = "Generic West End / Broadway-style receiving house".into();
    p.notes = "Benchmark large commercial house: deep stage, tall fly tower with a counterweight/automated \
flying system, raked stage, revolve, stage lift and a full orchestra pit lift."
        .into();
    p.capacity = 1200;
    p.pros_width = 12.5;
    p.pros_height = 8.0;
    p.stage_depth = 14.0;
    p.wing_width = 7.0;
    p.apron_depth = 1.5;
    p.grid_height = 24.0;
    p.fly_lines = 40;
    p.fly_system = "Automated / counterweight flying".into();
    p.rake_deg = 1.5;
    p.stage_height = 1.0;
    p.has_pit = true;
    p.pit_width = 12.0;
    p.pit_depth = 3.5;
    p.pit_is_lift = true;
    p.has_revolve = true;
    p.revolve_diameter = 10.0;
    p.revolve_z = -6.0;
    p.has_stage_lift = true;
    p.lift_width = 3.0;
    p.lift_depth = 1.5;
    p.lift_z = -11.5;
    p.house_width = 26.0;
    p.house_rise_per_row = 0.12;
    p.fly_gallery_height = 10.0;
    p.look = RoomLook::RedGilt;
    p.rear_gallery = true;
    p.cross_aisle_after = 8;
    p.perch_dist = 2.0;
    p.foh_boom_dist = 8.0;
    p.confidence = "Typical figures for a large proscenium house, not a specific building.".into();
    v.push(p);

    // Benchmark: flexible black box.
    let mut p = base("black_box", "Flexible Black Box (benchmark)");
    p.location = "Generic studio theatre".into();
    p.notes = "A black room with a pipe grid and movable seating risers on one side. No proscenium, no wings: \
masking is drapes on the grid. Everything is visible, so sightlines are about the seating risers."
        .into();
    p.kind = VenueKind::BlackBox;
    p.capacity = 100;
    p.pros_width = 11.0;
    p.pros_height = 6.0;
    p.stage_depth = 8.0;
    p.wing_width = 1.0;
    p.apron_depth = 0.0;
    p.grid_height = 0.0;
    p.pipe_height = 6.0;
    p.fly_lines = 6;
    p.fly_system = "Fixed pipe grid".into();
    p.stage_height = 0.0;
    p.has_crossover = false;
    p.has_house_steps = false;
    p.house_width = 12.0;
    p.house_rise_per_row = 0.35;
    p.confidence = "Typical studio figures.".into();
    v.push(p);

    // Custom.
    let mut p = base("custom", "Custom Stage");
    p.location = "Your own venue".into();
    p.notes = "Enter your own measurements in the Venue panel.".into();
    p.grid_height = 10.0;
    p.fly_system = "Your choice".into();
    p.confidence = "User-entered.".into();
    v.push(p);

    v
}

/// A standard set of battens for a venue.
pub fn default_battens(v: &Venue) -> Vec<Batten> {
    if v.open_end_stage && !v.has_fly_tower() {
        return end_stage_battens(v);
    }
    match base_key(v) {
        "capitol_tamworth" => return capitol_battens(v),
        "tas_hoskins" => return hoskins_battens(v),
        _ => {}
    }
    let mut out = Vec::new();
    let w = v.pros_width;
    let fly = v.has_fly_tower();
    let max = v.max_trim();
    let ph = v.pros_height;
    let usable = (v.stage_depth - 1.2).max(1.5);
    let mut add = |name: &str, kind: BattenKind, frac: f32, goods_h: f32, color: [u8; 3], height: f32| {
        out.push(Batten {
            name: name.into(),
            kind,
            dist: (0.3 + frac * usable).min(v.stage_depth - 0.4),
            height: height.min(max),
            goods_h,
            color,
            open: 1.0,
            visible: true,
            opening: 0.0,
            border_h: 0.0,
        });
    };
    let black = [24, 24, 28];
    if v.kind == VenueKind::Proscenium {
        add("House curtain", BattenKind::HouseCurtain, 0.0, ph + 0.4, [128, 20, 30], ph + 0.4);
    }
    let border_h = if fly { 1.8 } else { 0.9 };
    let border_trim = (ph + border_h - 0.3).min(max);
    add("LX 1", BattenKind::Lighting, 0.06, 0.0, [60, 60, 60], if fly { ph + 2.0 } else { max });
    add("Legs & Border 1", BattenKind::LegsAndBorder, 0.12, border_trim, black, border_trim);
    add("Scenery 1", BattenKind::Scenery, 0.2, 0.0, [80, 80, 80], max);
    if v.stage_depth > 4.5 {
        add("LX 2", BattenKind::Lighting, 0.33, 0.0, [60, 60, 60], if fly { ph + 2.0 } else { max });
        add("Legs & Border 2", BattenKind::LegsAndBorder, 0.4, border_trim, black, border_trim);
        add("Midstage tabs", BattenKind::Tabs, 0.5, ph + 0.6, black, max.min(ph + 3.0));
    }
    if v.stage_depth > 7.0 {
        // A painted drop only makes sense where it can fly out of sight.
        if fly {
            add("Scenery 2 (drop)", BattenKind::Drop, 0.58, ph + 0.6, [92, 120, 150], max);
        }
        add("LX 3", BattenKind::Lighting, 0.66, 0.0, [60, 60, 60], if fly { ph + 2.0 } else { max });
        add("Legs & Border 3", BattenKind::LegsAndBorder, 0.74, border_trim, black, border_trim);
    }
    add("LX 4 (cyc)", BattenKind::Lighting, 0.86, 0.0, [60, 60, 60], if fly { ph + 2.2 } else { max });
    let cyc_h = if fly { ph + 2.0 } else { (max - 0.1).max(2.5) };
    out.push(Batten {
        name: "Cyclorama".into(),
        kind: BattenKind::Cyclorama,
        dist: (v.stage_depth - 0.6).max(0.8),
        height: cyc_h,
        goods_h: cyc_h - 0.05,
        color: [222, 222, 228],
        open: 1.0,
        visible: true,
        opening: 0.0,
        border_h: 0.0,
    });
    if !fly {
        // Dead hung goods sit at their lowest useful trim.
        for b in out.iter_mut() {
            if matches!(b.kind, BattenKind::Drop | BattenKind::Tabs) {
                b.height = max;
                b.goods_h = b.goods_h.min(max - 0.05);
            }
        }
    }
    let _ = w;
    out
}

#[allow(clippy::too_many_arguments)]
fn line(name: &str, kind: BattenKind, dist: f32, height: f32, goods_h: f32, color: [u8; 3], opening: f32, border_h: f32) -> Batten {
    Batten { name: name.into(), kind, dist, height, goods_h, color, open: 1.0, visible: true, opening, border_h }
}

/// Capitol Theatre Tamworth: the venue's published hanging plot (distances from the setting
/// line) with its drapery sizes and standard trims.
fn capitol_battens(v: &Venue) -> Vec<Batten> {
    let max = v.max_trim();
    let black = [24, 24, 28];
    let gold = [178, 136, 52];
    let grey = [80, 80, 80];
    let lx = [60, 60, 60];
    let d = v.stage_depth;
    let mut out = vec![
        line("1 House curtain (gold, French action)", BattenKind::HouseCurtain, 0.45, 6.0, 6.0, gold, 0.0, 0.0),
        // False proscenium: 1.8 m legs, 2 m border trimmed at 5 m over a 10.6 m opening.
        line("2 False proscenium legs & border", BattenKind::LegsAndBorder, 0.6, 7.0, 7.0, black, 10.6, 2.0),
        line("3 LX 1", BattenKind::Lighting, 0.9, 5.7, 0.0, lx, 0.0, 0.0),
        line("4 Legs & border 2", BattenKind::LegsAndBorder, 2.5, 6.7, 6.7, black, 10.0, 2.0),
        line("5 Scenery batten 1", BattenKind::Scenery, 2.7, max, 0.0, grey, 0.0, 0.0),
        line("6 LX 2", BattenKind::Lighting, 3.15, 5.8, 0.0, lx, 0.0, 0.0),
        line("7 Black tabs 1", BattenKind::Tabs, 3.5, 7.0, 7.0, black, 0.0, 0.0),
        line("8 Scenery batten 1A (200 kg)", BattenKind::Scenery, 3.8, max, 0.0, grey, 0.0, 0.0),
        line("9 Legs & border 3", BattenKind::LegsAndBorder, 4.5, 6.7, 6.7, black, 10.0, 2.0),
        line("10 Scenery batten 2", BattenKind::Scenery, 4.7, max, 0.0, grey, 0.0, 0.0),
        line("11 LX 3", BattenKind::Lighting, 5.15, 5.8, 0.0, lx, 0.0, 0.0),
        line("12 Legs & border 4", BattenKind::LegsAndBorder, 6.5, 6.7, 6.7, black, 10.0, 2.0),
        line("13 Scenery batten 3", BattenKind::Scenery, 6.7, max, 0.0, grey, 0.0, 0.0),
        line("14 LX 4 (cyc)", BattenKind::Lighting, 7.0, 5.7, 0.0, lx, 0.0, 0.0),
        line("15 Black tabs 2", BattenKind::Tabs, 8.0, 7.0, 7.0, black, 0.0, 0.0),
        line("16 Scenery batten 4", BattenKind::Scenery, 8.2, max, 0.0, grey, 0.0, 0.0),
        line("17 White cyclorama", BattenKind::Cyclorama, 8.4, 7.0, 6.98, [226, 226, 230], 12.0, 0.0),
    ];
    for b in out.iter_mut() {
        b.dist = b.dist.min(d - 0.2);
        b.height = b.height.min(max);
    }
    out
}

/// TAS Hoskins Centre: four lighting trusses (two static, two on motorised winches), the
/// motorised projection screen on the proscenium truss and two movable masking tabs a side.
fn hoskins_battens(v: &Venue) -> Vec<Batten> {
    let max = v.max_trim();
    let black = [24, 24, 28];
    let lx = [60, 60, 60];
    let d = v.stage_depth;
    let trim = (v.pros_height - 0.15).min(max);
    let mut out = vec![
        line("Truss 1 – proscenium (static)", BattenKind::Lighting, 0.4, trim, 0.0, lx, 0.0, 0.0),
        // 200-inch 4:3 screen: 4.06 m x 3.05 m, flown out by default.
        Batten {
            visible: false,
            ..line("Projection screen (motorised, show when needed)", BattenKind::Drop, 0.55, trim, 3.05, [214, 214, 218], 4.06, 0.0)
        },
        line("Movable tabs 1 (masking)", BattenKind::Legs, 1.9, trim, trim - 0.02, black, 10.4, 0.0),
        line("Truss 2 (motorised winches)", BattenKind::Lighting, 3.1, trim, 0.0, lx, 0.0, 0.0),
        line("Movable tabs 2 (masking)", BattenKind::Legs, 4.3, trim, trim - 0.02, black, 10.0, 0.0),
        line("Truss 3 (motorised winches, position estimated)", BattenKind::Lighting, 5.0, trim, 0.0, lx, 0.0, 0.0),
        line("Truss 4 – rear wall (static, cyc)", BattenKind::Lighting, 6.7, trim, 0.0, lx, 0.0, 0.0),
        line("Rear blacks", BattenKind::Tabs, 7.1, trim, trim - 0.02, black, 0.0, 0.0),
    ];
    for b in out.iter_mut() {
        b.dist = b.dist.min(d - 0.15);
    }
    out
}

/// Open end stage (studio): one bar in the stage ceiling, black side masking and
/// black traverse curtains along the back wall. No house curtain, no flying.
fn end_stage_battens(v: &Venue) -> Vec<Batten> {
    let max = v.max_trim();
    let black = [22, 22, 26];
    let d = v.stage_depth;
    let mk = |name: &str, kind: BattenKind, dist: f32, height: f32, goods_h: f32, color: [u8; 3], open: f32| Batten {
        name: name.into(),
        kind,
        dist,
        height,
        goods_h,
        color,
        open,
        visible: true,
        opening: 0.0,
        border_h: 0.0,
    };
    vec![
        mk("Stage LX bar", BattenKind::Lighting, 0.35, max, 0.0, [60, 60, 60], 1.0),
        mk("Black legs (masking)", BattenKind::Legs, 0.5, max, max - 0.02, black, 1.0),
        mk("Scenery bar", BattenKind::Scenery, (d * 0.55).min(d - 0.8), max, 0.0, [80, 80, 80], 1.0),
        mk("Rear traverse (blacks)", BattenKind::Tabs, d - 0.2, max, max - 0.02, black, 1.0),
    ]
}

/// Lighting rig for an open end stage: two front bars over the house, a short bar in the
/// stage ceiling, side positions behind the masking and wall-mounted PARs in the house.
fn end_stage_lights(v: &Venue, battens: &[Batten]) -> Vec<Light> {
    let mut l = Vec::new();
    let w = v.pros_width;
    let hw = v.stage_width() * 0.5;
    let d = v.stage_depth;
    let warm = [1.0, 0.84, 0.66];
    let cool = [0.66, 0.78, 1.0];
    let white = [1.0, 0.96, 0.9];
    let fx = |name: String, kind: LightKind, pos: [f32; 3], pn: &str, target: [f32; 3], beam: f32, i: f32, c: [f32; 3]| Light {
        name,
        kind,
        mount: LightMount::Fixed { pos, position_name: pn.into() },
        target,
        beam_deg: beam,
        intensity: i,
        color: c,
    };
    // FOH 1: on the bulkhead just in front of the stage edge.
    let n1 = 8;
    for i in 0..n1 {
        let t = i as f32 / (n1 - 1) as f32;
        let x = -hw + 0.5 + (2.0 * hw - 1.0) * t;
        let end = i < 2 || i >= n1 - 2;
        let tx = (x * 0.75).clamp(-w * 0.45, w * 0.45);
        l.push(fx(
            format!("FOH 1.{}", i + 1),
            if end { LightKind::Profile } else { LightKind::Fresnel },
            [x, v.pros_height - 0.15, 0.55],
            "FOH bar 1",
            [tx, 0.9, -d * 0.45],
            if end { 26.0 } else { 44.0 },
            0.42,
            if i % 2 == 0 { warm } else { white },
        ));
    }
    // FOH 2: over the seats.
    let n2 = 8;
    let z2 = v.house_start_z() + 1.6;
    for i in 0..n2 {
        let t = i as f32 / (n2 - 1) as f32;
        let x = -hw + 0.6 + (2.0 * hw - 1.2) * t;
        let end = i == 0 || i == n2 - 1;
        let tx = (x * 0.6).clamp(-w * 0.45, w * 0.45);
        l.push(fx(
            format!("FOH 2.{}", i + 1),
            if end { LightKind::Profile } else { LightKind::Par },
            [x, v.pros_height + 0.3, z2],
            "FOH bar 2",
            [tx, 1.0, -d * 0.25],
            if end { 24.0 } else { 34.0 },
            0.38,
            if end { white } else if i % 2 == 0 { warm } else { cool },
        ));
    }
    // Stage ceiling bar: top/back light.
    if let Some(bi) = battens.iter().position(|b| b.kind == BattenKind::Lighting) {
        for i in 0..4 {
            let x = -w * 0.38 + w * 0.76 * (i as f32 / 3.0);
            l.push(Light {
                name: format!("Stage LX {}", i + 1),
                kind: LightKind::Fresnel,
                mount: LightMount::Batten { index: bi, x },
                target: [x, 0.0, -d * 0.6],
                beam_deg: 50.0,
                intensity: 0.4,
                color: if i % 2 == 0 { cool } else { warm },
            });
        }
    }
    // Side positions behind the masking (stage left is the low-ceiling side).
    for (side, sx) in [("SL", 1.0f32), ("SR", -1.0f32)] {
        let x = sx * (w * 0.5 + 0.6);
        let y = if sx > 0.0 { v.sl_soffit_height.max(2.0) - 0.2 } else { v.pros_height - 0.3 };
        l.push(fx(
            format!("{side} side"),
            LightKind::Profile,
            [x, y, -d * 0.35],
            &format!("{side} side pipe"),
            [-sx * w * 0.2, 1.2, -d * 0.4],
            30.0,
            0.35,
            if sx > 0.0 { cool } else { warm },
        ));
    }
    // LED PARs on the house-left wall.
    for k in 0..2 {
        let z = v.house_start_z() + 0.6 + k as f32 * 0.5;
        l.push(fx(
            format!("House wall PAR {}", k + 1),
            LightKind::Par,
            [-hw + 0.25, v.pros_height + 0.1, z],
            "House-left wall",
            [w * 0.15, 1.0, -d * 0.4],
            32.0,
            0.3,
            [0.85, 0.55, 1.0],
        ));
    }
    l
}

/// Height of the house floor (relative to the stage deck) at distance z in front of the setting line.
pub fn house_floor_at(v: &Venue, z: f32) -> f32 {
    let mut y = -v.stage_height;
    if v.house_rise_per_row > 0.0 {
        let start = v.house_start_z();
        for r in 0..v.rows() {
            if start + v.row_offset(r) <= z {
                y = -v.stage_height + v.row_rise(r);
            }
        }
    }
    y
}

/// A standard lighting rig: FOH, overhead washes, side booms and cyc floods.
pub fn default_lights(v: &Venue, battens: &[Batten]) -> Vec<Light> {
    if v.open_end_stage && !v.has_fly_tower() {
        return end_stage_lights(v, battens);
    }
    let mut l = Vec::new();
    let w = v.pros_width;
    let d = v.stage_depth;
    let warm = [1.0, 0.82, 0.62];
    let cool = [0.62, 0.75, 1.0];
    let white = [1.0, 0.96, 0.9];
    // FOH bridge / gallery in the house (or a plain bar just in front of the stage).
    let foh_z = if v.foh_dist > 0.0 { v.foh_dist } else { v.house_start_z() + 3.5 };
    let foh_name = if foh_z < v.house_start_z() {
        "FOH bar"
    } else if v.foh_dist > 0.0 && v.foh_dist < 10.0 {
        "FOH gallery bars"
    } else {
        "FOH bridge"
    };
    let foh_y = (v.pros_height + 1.0).max(house_floor_at(v, foh_z) + 4.0);
    let n = if w > 8.0 { 4 } else { 3 };
    for i in 0..n {
        let x = -w * 0.35 + w * 0.7 * (i as f32 / (n - 1) as f32);
        l.push(Light {
            name: format!("FOH {}", i + 1),
            kind: LightKind::Profile,
            mount: LightMount::Fixed { pos: [x * 1.2, foh_y, foh_z], position_name: foh_name.into() },
            target: [x, 1.2, -d * 0.2],
            beam_deg: 24.0,
            intensity: 0.8,
            color: if i % 2 == 0 { warm } else { white },
        });
    }
    // Side-wall perches near the stage and booms further back (cross light from the house).
    let wall_x = v.house_width * 0.5 - 0.35;
    for (dist, what, y_above, beam) in [(v.perch_dist, "Perch", 0.0f32, 26.0f32), (v.foh_boom_dist, "Box boom", 3.6, 22.0)] {
        if dist <= 0.0 {
            continue;
        }
        for (side, sx) in [("PS", -1.0f32), ("OP", 1.0f32)] {
            // Prompt side is stage left; in this model stage left is +X.
            let sx = -sx;
            let y = if y_above > 0.0 { house_floor_at(v, dist) + y_above } else { v.pros_height - 0.3 };
            l.push(Light {
                name: format!("{what} {side}"),
                kind: LightKind::Profile,
                mount: LightMount::Fixed { pos: [sx * wall_x, y, dist], position_name: format!("{what} {side} (house wall)") },
                target: [-sx * w * 0.2, 1.2, -d * 0.3],
                beam_deg: beam,
                intensity: 0.55,
                color: if sx > 0.0 { cool } else { warm },
            });
        }
    }
    // Overhead washes.
    let mut lx_count = 0;
    for (bi, b) in battens.iter().enumerate() {
        if b.kind != BattenKind::Lighting {
            continue;
        }
        lx_count += 1;
        let is_cyc = b.name.contains("cyc");
        if is_cyc {
            for i in 0..3 {
                let x = -w * 0.33 + w * 0.33 * i as f32;
                l.push(Light {
                    name: format!("Cyc flood {}", i + 1),
                    kind: LightKind::Flood,
                    mount: LightMount::Batten { index: bi, x },
                    target: [x, v.pros_height * 0.5, -(d - 0.6)],
                    beam_deg: 70.0,
                    intensity: 0.7,
                    color: [0.35, 0.5, 1.0],
                });
            }
            continue;
        }
        let m = if w > 8.0 { 3 } else { 2 };
        for i in 0..m {
            let x = -w * 0.3 + w * 0.6 * (i as f32 / (m - 1) as f32);
            l.push(Light {
                name: format!("LX{} wash {}", lx_count, i + 1),
                kind: LightKind::Fresnel,
                mount: LightMount::Batten { index: bi, x },
                target: [x, 0.0, -b.dist + 0.8],
                beam_deg: 48.0,
                intensity: 0.6,
                color: if lx_count % 2 == 1 { warm } else { cool },
            });
        }
    }
    // Side booms.
    if v.kind == VenueKind::Proscenium {
        let bx = w * 0.5 + (v.wing_width * 0.4).min(1.0);
        for (side, sx) in [("SL", 1.0f32), ("SR", -1.0f32)] {
            l.push(Light {
                name: format!("{side} boom"),
                kind: LightKind::Par,
                mount: LightMount::Fixed {
                    pos: [sx * bx, 2.2, -d * 0.3],
                    position_name: format!("{side} boom"),
                },
                target: [-sx * w * 0.15, 1.0, -d * 0.3],
                beam_deg: 30.0,
                intensity: 0.5,
                color: if sx > 0.0 { cool } else { warm },
            });
        }
    }
    l
}

pub fn fresh_state(v: &Venue) -> StageState {
    let battens = default_battens(v);
    let lights = default_lights(v, &battens);
    StageState {
        pieces: Vec::new(),
        battens,
        lights,
        revolve_angle: 0.0,
        pit_lift: if !v.has_pit {
            0.0
        } else if base_key(v) == "tas_hoskins" {
            // Pit floor at its 1200 mm setting.
            -(v.stage_height + 1.2)
        } else {
            -(v.stage_height + 1.5)
        },
        stage_lift: 0.0,
        house_lights: 0.1,
        notes: String::new(),
    }
}
