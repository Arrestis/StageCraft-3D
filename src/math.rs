//! Small 3D math helpers (vectors and 4x4 matrices, column-major).

use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub const fn v3(x: f32, y: f32, z: f32) -> V3 {
    V3 { x, y, z }
}

impl V3 {
    pub const ZERO: V3 = v3(0.0, 0.0, 0.0);
    pub fn dot(self, o: V3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    pub fn cross(self, o: V3) -> V3 {
        v3(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn norm(self) -> V3 {
        let l = self.len();
        if l > 1e-9 {
            self / l
        } else {
            v3(0.0, 1.0, 0.0)
        }
    }
    pub fn lerp(self, o: V3, t: f32) -> V3 {
        self + (o - self) * t
    }
    pub fn from_arr(a: [f32; 3]) -> V3 {
        v3(a[0], a[1], a[2])
    }
    pub fn arr(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
    /// Rotate around the vertical (Y) axis by `deg` degrees.
    pub fn rot_y(self, deg: f32) -> V3 {
        let (s, c) = deg.to_radians().sin_cos();
        v3(self.x * c + self.z * s, self.y, -self.x * s + self.z * c)
    }
}

impl Add for V3 {
    type Output = V3;
    fn add(self, o: V3) -> V3 {
        v3(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for V3 {
    type Output = V3;
    fn sub(self, o: V3) -> V3 {
        v3(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl Mul<f32> for V3 {
    type Output = V3;
    fn mul(self, s: f32) -> V3 {
        v3(self.x * s, self.y * s, self.z * s)
    }
}
impl Div<f32> for V3 {
    type Output = V3;
    fn div(self, s: f32) -> V3 {
        v3(self.x / s, self.y / s, self.z / s)
    }
}
impl Neg for V3 {
    type Output = V3;
    fn neg(self) -> V3 {
        v3(-self.x, -self.y, -self.z)
    }
}

/// Column-major 4x4 matrix: m[col][row].
#[derive(Clone, Copy, Debug)]
pub struct M4 {
    pub m: [[f32; 4]; 4],
}

impl M4 {
    pub fn identity() -> M4 {
        let mut m = [[0.0; 4]; 4];
        for (i, c) in m.iter_mut().enumerate() {
            c[i] = 1.0;
        }
        M4 { m }
    }

    pub fn mul(&self, o: &M4) -> M4 {
        let mut r = [[0.0f32; 4]; 4];
        for c in 0..4 {
            for row in 0..4 {
                let mut s = 0.0;
                for k in 0..4 {
                    s += self.m[k][row] * o.m[c][k];
                }
                r[c][row] = s;
            }
        }
        M4 { m: r }
    }

    pub fn transform4(&self, p: V3) -> [f32; 4] {
        let m = &self.m;
        [
            m[0][0] * p.x + m[1][0] * p.y + m[2][0] * p.z + m[3][0],
            m[0][1] * p.x + m[1][1] * p.y + m[2][1] * p.z + m[3][1],
            m[0][2] * p.x + m[1][2] * p.y + m[2][2] * p.z + m[3][2],
            m[0][3] * p.x + m[1][3] * p.y + m[2][3] * p.z + m[3][3],
        ]
    }

    pub fn look_at(eye: V3, target: V3, up: V3) -> M4 {
        let f = (target - eye).norm();
        let mut s = f.cross(up);
        if s.len() < 1e-5 {
            s = f.cross(v3(0.0, 0.0, -1.0));
        }
        let s = s.norm();
        let u = s.cross(f);
        let mut m = M4::identity();
        m.m[0][0] = s.x;
        m.m[1][0] = s.y;
        m.m[2][0] = s.z;
        m.m[0][1] = u.x;
        m.m[1][1] = u.y;
        m.m[2][1] = u.z;
        m.m[0][2] = -f.x;
        m.m[1][2] = -f.y;
        m.m[2][2] = -f.z;
        m.m[3][0] = -s.dot(eye);
        m.m[3][1] = -u.dot(eye);
        m.m[3][2] = f.dot(eye);
        m
    }

    pub fn perspective(fov_y_deg: f32, aspect: f32, near: f32, far: f32) -> M4 {
        let f = 1.0 / (fov_y_deg.to_radians() * 0.5).tan();
        let mut m = M4 { m: [[0.0; 4]; 4] };
        m.m[0][0] = f / aspect;
        m.m[1][1] = f;
        m.m[2][2] = (far + near) / (near - far);
        m.m[2][3] = -1.0;
        m.m[3][2] = 2.0 * far * near / (near - far);
        m
    }

    pub fn ortho(half_h: f32, aspect: f32, near: f32, far: f32) -> M4 {
        let half_w = half_h * aspect;
        let mut m = M4::identity();
        m.m[0][0] = 1.0 / half_w;
        m.m[1][1] = 1.0 / half_h;
        m.m[2][2] = -2.0 / (far - near);
        m.m[3][2] = -(far + near) / (far - near);
        m
    }

    pub fn inverse(&self) -> M4 {
        // Flatten column-major.
        let a: Vec<f32> = (0..16).map(|i| self.m[i / 4][i % 4]).collect();
        let mut inv = [0.0f32; 16];
        inv[0] = a[5] * a[10] * a[15] - a[5] * a[11] * a[14] - a[9] * a[6] * a[15]
            + a[9] * a[7] * a[14]
            + a[13] * a[6] * a[11]
            - a[13] * a[7] * a[10];
        inv[4] = -a[4] * a[10] * a[15] + a[4] * a[11] * a[14] + a[8] * a[6] * a[15]
            - a[8] * a[7] * a[14]
            - a[12] * a[6] * a[11]
            + a[12] * a[7] * a[10];
        inv[8] = a[4] * a[9] * a[15] - a[4] * a[11] * a[13] - a[8] * a[5] * a[15]
            + a[8] * a[7] * a[13]
            + a[12] * a[5] * a[11]
            - a[12] * a[7] * a[9];
        inv[12] = -a[4] * a[9] * a[14] + a[4] * a[10] * a[13] + a[8] * a[5] * a[14]
            - a[8] * a[6] * a[13]
            - a[12] * a[5] * a[10]
            + a[12] * a[6] * a[9];
        inv[1] = -a[1] * a[10] * a[15] + a[1] * a[11] * a[14] + a[9] * a[2] * a[15]
            - a[9] * a[3] * a[14]
            - a[13] * a[2] * a[11]
            + a[13] * a[3] * a[10];
        inv[5] = a[0] * a[10] * a[15] - a[0] * a[11] * a[14] - a[8] * a[2] * a[15]
            + a[8] * a[3] * a[14]
            + a[12] * a[2] * a[11]
            - a[12] * a[3] * a[10];
        inv[9] = -a[0] * a[9] * a[15] + a[0] * a[11] * a[13] + a[8] * a[1] * a[15]
            - a[8] * a[3] * a[13]
            - a[12] * a[1] * a[11]
            + a[12] * a[3] * a[9];
        inv[13] = a[0] * a[9] * a[14] - a[0] * a[10] * a[13] - a[8] * a[1] * a[14]
            + a[8] * a[2] * a[13]
            + a[12] * a[1] * a[10]
            - a[12] * a[2] * a[9];
        inv[2] = a[1] * a[6] * a[15] - a[1] * a[7] * a[14] - a[5] * a[2] * a[15]
            + a[5] * a[3] * a[14]
            + a[13] * a[2] * a[7]
            - a[13] * a[3] * a[6];
        inv[6] = -a[0] * a[6] * a[15] + a[0] * a[7] * a[14] + a[4] * a[2] * a[15]
            - a[4] * a[3] * a[14]
            - a[12] * a[2] * a[7]
            + a[12] * a[3] * a[6];
        inv[10] = a[0] * a[5] * a[15] - a[0] * a[7] * a[13] - a[4] * a[1] * a[15]
            + a[4] * a[3] * a[13]
            + a[12] * a[1] * a[7]
            - a[12] * a[3] * a[5];
        inv[14] = -a[0] * a[5] * a[14] + a[0] * a[6] * a[13] + a[4] * a[1] * a[14]
            - a[4] * a[2] * a[13]
            - a[12] * a[1] * a[6]
            + a[12] * a[2] * a[5];
        inv[3] = -a[1] * a[6] * a[11] + a[1] * a[7] * a[10] + a[5] * a[2] * a[11]
            - a[5] * a[3] * a[10]
            - a[9] * a[2] * a[7]
            + a[9] * a[3] * a[6];
        inv[7] = a[0] * a[6] * a[11] - a[0] * a[7] * a[10] - a[4] * a[2] * a[11]
            + a[4] * a[3] * a[10]
            + a[8] * a[2] * a[7]
            - a[8] * a[3] * a[6];
        inv[11] = -a[0] * a[5] * a[11] + a[0] * a[7] * a[9] + a[4] * a[1] * a[11]
            - a[4] * a[3] * a[9]
            - a[8] * a[1] * a[7]
            + a[8] * a[3] * a[5];
        inv[15] = a[0] * a[5] * a[10] - a[0] * a[6] * a[9] - a[4] * a[1] * a[10]
            + a[4] * a[2] * a[9]
            + a[8] * a[1] * a[6]
            - a[8] * a[2] * a[5];
        let det = a[0] * inv[0] + a[1] * inv[4] + a[2] * inv[8] + a[3] * inv[12];
        let d = if det.abs() < 1e-12 { 0.0 } else { 1.0 / det };
        let mut r = [[0.0f32; 4]; 4];
        for i in 0..16 {
            r[i / 4][i % 4] = inv[i] * d;
        }
        M4 { m: r }
    }
}

pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
