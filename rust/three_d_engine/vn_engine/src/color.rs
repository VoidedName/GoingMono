#[derive(Copy, Clone)]
pub struct RGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub const WHITE: RGBA = RGBA {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
pub const BLACK: RGBA = RGBA {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
pub const RED: RGBA = RGBA {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
pub const GREEN: RGBA = RGBA {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
pub const BLUE: RGBA = RGBA {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
pub const YELLOW: RGBA = RGBA {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
pub const CYAN: RGBA = RGBA {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
pub const VIOLET: RGBA = RGBA {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
