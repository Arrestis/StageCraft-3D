//! Project data: venue, rigging, lights, set pieces and scene states.
//!
//! Coordinates (metres): X runs across the stage (+X = stage left = house right),
//! Y is up (0 = stage deck at the setting line), Z runs up/down stage
//! (0 = setting line behind the proscenium, negative = upstage, positive = towards the audience).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VenueKind {
    Proscenium,
    BlackBox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurtainStyle {
    /// Two halves that draw apart to the sides.
    Traveller,
    /// One curtain that flies straight up out of view.
    Guillotine,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Venue {
    pub key: String,
    pub name: String,
    pub location: String,
    pub notes: String,
    pub kind: VenueKind,
    pub capacity: u32,
    /// Proscenium (or playing-area) opening width.
    pub pros_width: f32,
    pub pros_height: f32,
    /// Setting line to back wall.
    pub stage_depth: f32,
    /// Offstage wing width on each side of the opening.
    pub wing_width: f32,
    /// Forestage in front of the setting line.
    pub apron_depth: f32,
    /// Stage-floor-to-grid height; 0 = no fly tower (dead-hung pipes only).
    pub grid_height: f32,
    /// Highest a dead-hung pipe can go when there is no fly tower.
    pub pipe_height: f32,
    pub fly_lines: u32,
    pub fly_system: String,
    /// Stage rake in degrees (rises upstage).
    pub rake_deg: f32,
    /// Stage floor height above the front of the auditorium.
    pub stage_height: f32,
    pub has_pit: bool,
    pub pit_width: f32,
    pub pit_depth: f32,
    pub pit_is_lift: bool,
    pub has_revolve: bool,
    pub revolve_diameter: f32,
    pub revolve_z: f32,
    pub has_stage_lift: bool,
    pub lift_width: f32,
    pub lift_depth: f32,
    pub lift_x: f32,
    pub lift_z: f32,
    pub has_crossover: bool,
    pub has_house_steps: bool,
    pub curtain_style: CurtainStyle,
    pub house_width: f32,
    /// Rise per seat row in the auditorium (0 = flat floor).
    pub house_rise_per_row: f32,
    pub fly_gallery_height: f32,
    /// How much of this preset is confirmed by published specs.
    pub confidence: String,
    /// Open end stage: no proscenium arch, just a low lighting bulkhead across the front
    /// and a flat ceiling over the stage at `pros_height`.
    #[serde(default)]
    pub open_end_stage: bool,
    /// Lowered ceiling (soffit) on the stage-left side: width from the SL wall, and its
    /// height at the wall (it slopes up to the main ceiling). 0 = none.
    #[serde(default)]
    pub sl_soffit_width: f32,
    #[serde(default)]
    pub sl_soffit_height: f32,
    /// Black deck, charcoal walls, grey timber house walls (studio look).
    #[serde(default)]
    pub dark_room: bool,
    /// Seats per row (0 = worked out from the house width).
    #[serde(default)]
    pub seats_per_row_fixed: u32,
    /// Aisle layout: 0 = centre aisle(s); 1 = one side aisle on house right (stage-left side).
    #[serde(default)]
    pub side_aisle: bool,
    /// Draw a control box (bio box) at the back of the house.
    #[serde(default)]
    pub control_box: bool,
    /// Sound, lighting and comms equipment in the venue.
    #[serde(default)]
    pub tech_notes: String,
    /// Colours and finishes of the room.
    #[serde(default)]
    pub look: RoomLook,
    /// Raised side galleries (balconies) along both house walls.
    #[serde(default)]
    pub side_galleries: bool,
    /// A balcony / gallery across the back of the house.
    #[serde(default)]
    pub rear_gallery: bool,
    /// Aisle stairs on both sides of one central seating block.
    #[serde(default)]
    pub aisles_both_sides: bool,
    /// A cross aisle after this many rows (0 = none).
    #[serde(default)]
    pub cross_aisle_after: u32,
    /// Front-of-house lighting bridge / gallery, distance from the setting line (0 = automatic).
    #[serde(default)]
    pub foh_dist: f32,
    /// Side-wall perch positions near the stage, distance from the setting line (0 = none).
    #[serde(default)]
    pub perch_dist: f32,
    /// Side-wall FOH booms further back in the house, distance from the setting line (0 = none).
    #[serde(default)]
    pub foh_boom_dist: f32,
    /// Projection screens either side of the stage opening.
    #[serde(default)]
    pub side_screens: bool,
}

/// The finishes of a room: walls, proscenium, seats and floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RoomLook {
    /// Maroon walls, red seats and a gilded proscenium.
    #[default]
    Classic,
    /// Black deck, charcoal walls, grey timber house (studio).
    Studio,
    /// Converted church: dark oak panelling, cream walls, timber roof, red velvet seats.
    TimberChurch,
    /// Teal walls with gold trim, light timber proscenium, red-carpeted aisle stairs.
    TealGold,
    /// Large hall: timber walls and slatted ceiling, black-draped stage, dark seats.
    TimberHall,
    /// Grand house: deep red and gold.
    RedGilt,
}

impl RoomLook {
    pub const ALL: [RoomLook; 6] = [
        RoomLook::Classic,
        RoomLook::Studio,
        RoomLook::TimberChurch,
        RoomLook::TealGold,
        RoomLook::TimberHall,
        RoomLook::RedGilt,
    ];
    pub fn label(self) -> &'static str {
        match self {
            RoomLook::Classic => "Classic (maroon & gold)",
            RoomLook::Studio => "Studio (black & charcoal)",
            RoomLook::TimberChurch => "Converted church (oak panels)",
            RoomLook::TealGold => "Teal & gold",
            RoomLook::TimberHall => "Timber hall",
            RoomLook::RedGilt => "Grand red & gilt",
        }
    }
}

/// Colours used to draw a room.
pub struct Palette {
    pub deck: [u8; 3],
    pub stage_wall: [u8; 3],
    pub pros: [u8; 3],
    pub trim: [u8; 3],
    pub house_wall: [u8; 3],
    /// Lower wall panelling up to 1.4 m.
    pub wainscot: Option<[u8; 3]>,
    /// A thin rail along the house walls.
    pub rail: Option<[u8; 3]>,
    pub carpet: [u8; 3],
    pub seat: [u8; 3],
    pub riser: [u8; 3],
    pub tread: [u8; 3],
    pub steps: [u8; 3],
    pub ceiling: Option<[u8; 3]>,
    /// Individual seats with frames (rather than long seat blocks).
    pub separate_seats: bool,
}

impl Venue {
    pub fn palette(&self) -> Palette {
        let look = if self.dark_room && self.look == RoomLook::Classic { RoomLook::Studio } else { self.look };
        let mut p = Palette {
            deck: [66, 58, 52],
            stage_wall: [46, 45, 50],
            pros: [92, 30, 38],
            trim: [168, 134, 72],
            house_wall: [74, 46, 50],
            wainscot: None,
            rail: None,
            carpet: [58, 30, 34],
            seat: [128, 26, 36],
            riser: [44, 36, 38],
            tread: [164, 122, 74],
            steps: [54, 48, 44],
            ceiling: None,
            separate_seats: false,
        };
        match look {
            RoomLook::Classic => {}
            RoomLook::Studio => {
                p.deck = [34, 34, 38];
                p.stage_wall = [54, 54, 58];
                p.house_wall = [96, 96, 100];
                p.carpet = [40, 38, 42];
                p.seat = [176, 30, 36];
                p.riser = [46, 44, 48];
                p.steps = [30, 30, 34];
                p.separate_seats = true;
            }
            RoomLook::TimberChurch => {
                p.deck = [58, 50, 44];
                p.stage_wall = [40, 38, 40];
                p.pros = [98, 66, 38];
                p.trim = [130, 90, 50];
                p.house_wall = [212, 198, 166];
                p.wainscot = Some([92, 60, 36]);
                p.carpet = [96, 24, 30];
                p.seat = [158, 22, 32];
                p.riser = [76, 40, 34];
                p.tread = [120, 84, 52];
                p.steps = [60, 44, 34];
                p.ceiling = Some([104, 70, 42]);
                p.separate_seats = true;
            }
            RoomLook::TealGold => {
                p.deck = [36, 34, 36];
                p.stage_wall = [30, 30, 34];
                p.pros = [178, 138, 90];
                p.trim = [206, 168, 82];
                p.house_wall = [24, 92, 112];
                p.rail = Some([206, 168, 82]);
                p.carpet = [150, 32, 42];
                p.seat = [128, 24, 36];
                p.riser = [150, 32, 42];
                p.tread = [204, 164, 92];
                p.steps = [40, 36, 38];
                p.ceiling = Some([40, 46, 52]);
                p.separate_seats = true;
            }
            RoomLook::TimberHall => {
                p.deck = [44, 40, 38];
                p.stage_wall = [26, 26, 30];
                p.pros = [40, 38, 42];
                p.trim = [150, 120, 80];
                p.house_wall = [186, 148, 102];
                p.carpet = [126, 100, 74];
                p.seat = [38, 38, 44];
                p.riser = [110, 92, 74];
                p.tread = [172, 132, 86];
                p.steps = [44, 42, 46];
                p.ceiling = Some([150, 112, 72]);
            }
            RoomLook::RedGilt => {
                p.pros = [118, 24, 34];
                p.trim = [204, 164, 72];
                p.house_wall = [104, 28, 36];
                p.rail = Some([204, 164, 72]);
                p.carpet = [92, 20, 28];
                p.seat = [168, 28, 40];
                p.riser = [70, 20, 26];
                p.ceiling = Some([86, 30, 34]);
            }
        }
        p
    }
    /// Distance from the first row to the front of row r (includes any cross aisle).
    pub fn row_offset(&self, r: u32) -> f32 {
        let gap = if self.cross_aisle_after > 0 && r >= self.cross_aisle_after { 1.2 } else { 0.0 };
        r as f32 * 0.92 + gap
    }
    /// Floor height (relative to the house floor at the front) of row r.
    pub fn row_rise(&self, r: u32) -> f32 {
        r as f32 * self.house_rise_per_row
    }
    pub fn stage_width(&self) -> f32 {
        self.pros_width + 2.0 * self.wing_width
    }
    pub fn has_fly_tower(&self) -> bool {
        self.grid_height > 0.5
    }
    /// Highest point a batten pipe can reach.
    pub fn max_trim(&self) -> f32 {
        if self.has_fly_tower() {
            (self.grid_height - 0.6).max(2.0)
        } else {
            self.pipe_height.max(2.0)
        }
    }
    /// Height of the stage deck surface at upstage distance z (rake rises upstage).
    pub fn deck_y(&self, z: f32) -> f32 {
        if z < 0.0 {
            (-z).min(self.stage_depth) * self.rake_deg.to_radians().tan()
        } else {
            0.0
        }
    }
    pub fn house_start_z(&self) -> f32 {
        self.apron_depth + if self.has_pit { self.pit_depth } else { 0.8 } + 0.6
    }
    pub fn seats_per_row(&self) -> u32 {
        if self.seats_per_row_fixed > 0 {
            return self.seats_per_row_fixed;
        }
        (((self.house_width - 3.0).max(3.0)) / 0.52) as u32
    }
    /// Ceiling height over the stage at offset x, where the room has a low ceiling
    /// (open end stage). None = fly tower / tall stage house.
    pub fn ceiling_at(&self, x: f32) -> Option<f32> {
        if !self.open_end_stage {
            return None;
        }
        let hw = self.stage_width() * 0.5;
        let main = self.pros_height;
        if self.sl_soffit_width > 0.05 && self.sl_soffit_height > 0.5 {
            let inner = hw - self.sl_soffit_width;
            if x > inner {
                let t = ((x - inner) / self.sl_soffit_width).clamp(0.0, 1.0);
                return Some(main + (self.sl_soffit_height - main) * t);
            }
        }
        Some(main)
    }
    pub fn rows(&self) -> u32 {
        let spr = self.seats_per_row().max(1);
        ((self.capacity + spr - 1) / spr).max(2)
    }
    pub fn house_depth(&self) -> f32 {
        self.row_offset(self.rows()) + 2.5 + if self.rear_gallery { 0.8 } else { 0.0 }
    }
    pub fn tower_height(&self) -> f32 {
        if self.open_end_stage && !self.has_fly_tower() {
            self.pros_height
        } else if self.has_fly_tower() {
            self.grid_height + 1.5
        } else {
            self.pipe_height + 1.2
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattenKind {
    HouseCurtain,
    Lighting,
    Border,
    Legs,
    LegsAndBorder,
    Drop,
    Tabs,
    Cyclorama,
    Scenery,
}

impl BattenKind {
    pub fn label(self) -> &'static str {
        match self {
            BattenKind::HouseCurtain => "House curtain",
            BattenKind::Lighting => "Lighting bar",
            BattenKind::Border => "Border",
            BattenKind::Legs => "Legs",
            BattenKind::LegsAndBorder => "Legs & border",
            BattenKind::Drop => "Painted drop",
            BattenKind::Tabs => "Black tabs",
            BattenKind::Cyclorama => "Cyclorama",
            BattenKind::Scenery => "Scenery bar",
        }
    }
    pub const ALL: [BattenKind; 9] = [
        BattenKind::HouseCurtain,
        BattenKind::Lighting,
        BattenKind::Border,
        BattenKind::Legs,
        BattenKind::LegsAndBorder,
        BattenKind::Drop,
        BattenKind::Tabs,
        BattenKind::Cyclorama,
        BattenKind::Scenery,
    ];
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Batten {
    pub name: String,
    pub kind: BattenKind,
    /// Distance upstage of the setting line (positive number).
    pub dist: f32,
    /// Pipe height above the deck (the trim).
    pub height: f32,
    /// Height of the hanging goods (border, drop, legs...).
    pub goods_h: f32,
    pub color: [u8; 3],
    /// For curtains/tabs: 0 = closed, 1 = fully open.
    pub open: f32,
    pub visible: bool,
    /// Masking opening between the legs, or the width of a drop / screen (0 = automatic).
    #[serde(default)]
    pub opening: f32,
    /// Border height for legs & border lines (0 = automatic).
    #[serde(default)]
    pub border_h: f32,
}

impl Batten {
    /// Trim where the goods just touch the deck.
    pub fn in_trim(&self) -> f32 {
        match self.kind {
            BattenKind::Lighting | BattenKind::Scenery => 4.0,
            BattenKind::Border => self.goods_h + 3.0,
            _ => self.goods_h + 0.02,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightKind {
    Profile,
    Fresnel,
    Par,
    Flood,
}

impl LightKind {
    pub fn label(self) -> &'static str {
        match self {
            LightKind::Profile => "Profile spot",
            LightKind::Fresnel => "Fresnel wash",
            LightKind::Par => "PAR can",
            LightKind::Flood => "Cyc flood",
        }
    }
    pub fn default_beam(self) -> f32 {
        match self {
            LightKind::Profile => 26.0,
            LightKind::Fresnel => 45.0,
            LightKind::Par => 30.0,
            LightKind::Flood => 80.0,
        }
    }
    pub const ALL: [LightKind; 4] = [LightKind::Profile, LightKind::Fresnel, LightKind::Par, LightKind::Flood];
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LightMount {
    /// Hung on a batten at offset x.
    Batten { index: usize, x: f32 },
    /// Fixed position: FOH bridge, booms, ladders.
    Fixed { pos: [f32; 3], position_name: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Light {
    pub name: String,
    pub kind: LightKind,
    pub mount: LightMount,
    pub target: [f32; 3],
    pub beam_deg: f32,
    pub intensity: f32,
    pub color: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    Box,
    Cylinder,
    Cone,
    Sphere,
    /// Wedge rising towards the back (-z).
    Ramp,
    /// Solid stair unit rising towards the back (-z).
    Stairs(u8),
    /// The panel above a semicircular arch opening.
    ArchTop,
}

impl Shape {
    pub fn label(self) -> &'static str {
        match self {
            Shape::Box => "Box / flat",
            Shape::Cylinder => "Cylinder / column",
            Shape::Cone => "Cone",
            Shape::Sphere => "Sphere / rock",
            Shape::Ramp => "Ramp / wedge",
            Shape::Stairs(_) => "Stairs",
            Shape::ArchTop => "Arch top",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Material {
    PaintedWood,
    Timber,
    Fabric,
    Metal,
    Stone,
    Foliage,
    Masking,
    Glass,
}

impl Material {
    pub fn label(self) -> &'static str {
        match self {
            Material::PaintedWood => "Painted wood",
            Material::Timber => "Natural timber",
            Material::Fabric => "Upholstery / fabric",
            Material::Metal => "Metal",
            Material::Stone => "Stone / plaster",
            Material::Foliage => "Foliage",
            Material::Masking => "Black masking",
            Material::Glass => "Glass",
        }
    }
    pub fn default_color(self) -> [u8; 3] {
        match self {
            Material::PaintedWood => [196, 186, 160],
            Material::Timber => [140, 96, 60],
            Material::Fabric => [120, 60, 70],
            Material::Metal => [150, 152, 160],
            Material::Stone => [160, 155, 145],
            Material::Foliage => [70, 120, 60],
            Material::Masking => [28, 28, 32],
            Material::Glass => [170, 205, 225],
        }
    }
    pub fn specular(self) -> f32 {
        match self {
            Material::Metal => 0.55,
            Material::Glass => 0.7,
            Material::PaintedWood => 0.15,
            Material::Timber => 0.2,
            _ => 0.03,
        }
    }
    pub const ALL: [Material; 8] = [
        Material::PaintedWood,
        Material::Timber,
        Material::Fabric,
        Material::Metal,
        Material::Stone,
        Material::Foliage,
        Material::Masking,
        Material::Glass,
    ];
}

/// One building block of a set piece, in the piece's own space (metres).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Part {
    pub shape: Shape,
    /// Centre of the base (x, z) and bottom (y).
    pub offset: [f32; 3],
    /// Width (x), height (y), depth (z).
    pub size: [f32; 3],
    pub rot: f32,
    /// None = the piece's main colour.
    pub color: Option<[u8; 3]>,
    pub material: Option<Material>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibItem {
    pub name: String,
    pub category: String,
    pub material: Material,
    pub color: [u8; 3],
    pub parts: Vec<Part>,
}

impl LibItem {
    /// Bounding size (w, h, d) of all parts.
    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        parts_bounds(&self.parts)
    }
}

pub fn parts_bounds(parts: &[Part]) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for p in parts {
        let (s, c) = p.rot.to_radians().sin_cos();
        let hw = p.size[0] * 0.5;
        let hd = p.size[2] * 0.5;
        for (cx, cz) in [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)] {
            let x = p.offset[0] + cx * c + cz * s;
            let z = p.offset[2] - cx * s + cz * c;
            lo[0] = lo[0].min(x);
            hi[0] = hi[0].max(x);
            lo[2] = lo[2].min(z);
            hi[2] = hi[2].max(z);
        }
        lo[1] = lo[1].min(p.offset[1]);
        hi[1] = hi[1].max(p.offset[1] + p.size[1]);
    }
    if parts.is_empty() {
        return ([0.0; 3], [1.0; 3]);
    }
    (lo, hi)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Piece {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub parts: Vec<Part>,
    /// Natural size of the parts (w, h, d).
    pub base_size: [f32; 3],
    /// Current size (w, h, d): parts are stretched to fit.
    pub size: [f32; 3],
    /// x, elevation above whatever it stands on, z.
    pub pos: [f32; 3],
    pub rot: f32,
    pub material: Material,
    pub color: [u8; 3],
    pub visible: bool,
    pub locked: bool,
    /// Rides the revolve (position then relative to the revolve centre).
    pub on_revolve: bool,
    /// Hung from a batten (flies with it).
    pub flown_on: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageState {
    pub pieces: Vec<Piece>,
    pub battens: Vec<Batten>,
    pub lights: Vec<Light>,
    pub revolve_angle: f32,
    /// Orchestra pit lift height relative to the stage deck (0 = stage level).
    pub pit_lift: f32,
    /// Stage lift height relative to the deck.
    pub stage_lift: f32,
    /// House light level for cue mode, 0..1.
    pub house_lights: f32,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneState {
    pub name: String,
    pub state: StageState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub app: String,
    pub version: u32,
    pub name: String,
    pub venue: Venue,
    pub scenes: Vec<SceneState>,
    pub current: usize,
    pub custom_library: Vec<LibItem>,
    pub next_id: u32,
}
