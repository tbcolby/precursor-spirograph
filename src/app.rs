//! State machine for Spirograph Engine.
//!
//! States:
//!   Menu      — choose curve type and parameters
//!   Drawing   — live curve rendering
//!   Gallery   — preset patterns

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::math;

const KEY_UP: char = '\u{F700}';
const KEY_DOWN: char = '\u{F701}';
const KEY_LEFT: char = '\u{F702}';
const KEY_RIGHT: char = '\u{F703}';
const KEY_ENTER: char = '\u{000D}';
const KEY_BACKSPACE: char = '\u{0008}';
const KEY_MENU: char = '\u{2234}';

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Menu,
    Drawing,
    Gallery,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveType {
    Hypotrochoid,
    Epitrochoid,
}

impl CurveType {
    pub fn label(&self) -> &'static str {
        match self {
            CurveType::Hypotrochoid => "Hypotrochoid",
            CurveType::Epitrochoid => "Epitrochoid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuField {
    CurveType,
    BigR,
    SmallR,
    PenD,
    Draw,
    Gallery,
}

/// A preset pattern.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    pub name: &'static str,
    pub curve: CurveType,
    pub big_r: i32,
    pub small_r: i32,
    pub pen_d: i32,
}

pub const PRESETS: &[Preset] = &[
    Preset { name: "Classic Star", curve: CurveType::Hypotrochoid, big_r: 100, small_r: 40, pen_d: 30 },
    Preset { name: "Pentagon Flower", curve: CurveType::Hypotrochoid, big_r: 100, small_r: 60, pen_d: 50 },
    Preset { name: "Triangle Rose", curve: CurveType::Hypotrochoid, big_r: 90, small_r: 30, pen_d: 25 },
    Preset { name: "Seven-Point", curve: CurveType::Hypotrochoid, big_r: 105, small_r: 15, pen_d: 12 },
    Preset { name: "Cardioid", curve: CurveType::Epitrochoid, big_r: 50, small_r: 50, pen_d: 50 },
    Preset { name: "Limaçon", curve: CurveType::Epitrochoid, big_r: 60, small_r: 40, pen_d: 50 },
    Preset { name: "Nephroid", curve: CurveType::Epitrochoid, big_r: 50, small_r: 25, pen_d: 25 },
    Preset { name: "Astroid", curve: CurveType::Hypotrochoid, big_r: 100, small_r: 25, pen_d: 25 },
    Preset { name: "Deltoid", curve: CurveType::Hypotrochoid, big_r: 90, small_r: 30, pen_d: 30 },
    Preset { name: "Spirograph #42", curve: CurveType::Hypotrochoid, big_r: 96, small_r: 36, pen_d: 24 },
];

pub struct SpirographApp {
    pub state: AppState,
    pub needs_redraw: bool,

    // Menu
    pub menu_field: MenuField,
    pub curve_type: CurveType,
    pub big_r: i32,
    pub small_r: i32,
    pub pen_d: i32,

    // Drawing - precomputed points
    pub points: Vec<(i16, i16)>,

    // Gallery
    pub gallery_cursor: usize,
}

impl SpirographApp {
    pub fn new() -> Self {
        Self {
            state: AppState::Menu,
            needs_redraw: true,
            menu_field: MenuField::CurveType,
            curve_type: CurveType::Hypotrochoid,
            big_r: 100,
            small_r: 40,
            pen_d: 30,
            points: Vec::new(),
            gallery_cursor: 0,
        }
    }

    fn compute_curve(&mut self) {
        self.points.clear();
        let period = math::curve_period(self.big_r, self.small_r);
        let total_steps = (period * 1024).min(16384); // Cap iterations

        let cx: i16 = 168; // center x (336/2)
        let cy: i16 = 253; // center y (approx middle of content area)

        for step in 0..total_steps {
            let t = step; // angle in 1/1024 units
            let (x, y) = match self.curve_type {
                CurveType::Hypotrochoid => (
                    math::hypotrochoid_x(t, self.big_r, self.small_r, self.pen_d),
                    math::hypotrochoid_y(t, self.big_r, self.small_r, self.pen_d),
                ),
                CurveType::Epitrochoid => (
                    math::epitrochoid_x(t, self.big_r, self.small_r, self.pen_d),
                    math::epitrochoid_y(t, self.big_r, self.small_r, self.pen_d),
                ),
            };

            // Convert fixed-point to screen coords
            // Scale to fit: max curve extent is roughly R+d
            let max_extent = (self.big_r + self.pen_d).max(1);
            let scale = 140; // pixels for max extent
            let sx = cx + ((x as i64 * scale as i64 / max_extent as i64) as i16);
            let sy = cy + ((y as i64 * scale as i64 / max_extent as i64) as i16);

            self.points.push((sx, sy));
        }
    }

    pub fn handle_key(&mut self, key: char) -> bool {
        self.needs_redraw = true;
        match self.state {
            AppState::Menu => self.handle_menu(key),
            AppState::Drawing => self.handle_drawing(key),
            AppState::Gallery => self.handle_gallery(key),
        }
    }

    fn handle_menu(&mut self, key: char) -> bool {
        match key {
            KEY_MENU => return false,
            KEY_UP => {
                self.menu_field = match self.menu_field {
                    MenuField::CurveType => MenuField::Gallery,
                    MenuField::BigR => MenuField::CurveType,
                    MenuField::SmallR => MenuField::BigR,
                    MenuField::PenD => MenuField::SmallR,
                    MenuField::Draw => MenuField::PenD,
                    MenuField::Gallery => MenuField::Draw,
                };
            }
            KEY_DOWN => {
                self.menu_field = match self.menu_field {
                    MenuField::CurveType => MenuField::BigR,
                    MenuField::BigR => MenuField::SmallR,
                    MenuField::SmallR => MenuField::PenD,
                    MenuField::PenD => MenuField::Draw,
                    MenuField::Draw => MenuField::Gallery,
                    MenuField::Gallery => MenuField::CurveType,
                };
            }
            KEY_LEFT | KEY_RIGHT => {
                let delta = if key == KEY_RIGHT { 1 } else { -1 };
                match self.menu_field {
                    MenuField::CurveType => {
                        self.curve_type = match self.curve_type {
                            CurveType::Hypotrochoid => CurveType::Epitrochoid,
                            CurveType::Epitrochoid => CurveType::Hypotrochoid,
                        };
                    }
                    MenuField::BigR => {
                        self.big_r = (self.big_r + delta * 5).max(10).min(200);
                    }
                    MenuField::SmallR => {
                        self.small_r = (self.small_r + delta * 5).max(5).min(self.big_r - 5);
                    }
                    MenuField::PenD => {
                        self.pen_d = (self.pen_d + delta * 5).max(5).min(100);
                    }
                    _ => {}
                }
            }
            KEY_ENTER => match self.menu_field {
                MenuField::Draw => {
                    self.compute_curve();
                    self.state = AppState::Drawing;
                }
                MenuField::Gallery => {
                    self.state = AppState::Gallery;
                }
                MenuField::CurveType => {
                    self.curve_type = match self.curve_type {
                        CurveType::Hypotrochoid => CurveType::Epitrochoid,
                        CurveType::Epitrochoid => CurveType::Hypotrochoid,
                    };
                }
                _ => {}
            },
            _ => {}
        }
        true
    }

    fn handle_drawing(&mut self, key: char) -> bool {
        match key {
            KEY_MENU | KEY_LEFT | KEY_ENTER => {
                self.state = AppState::Menu;
            }
            _ => {}
        }
        true
    }

    fn handle_gallery(&mut self, key: char) -> bool {
        match key {
            KEY_MENU | KEY_LEFT => {
                self.state = AppState::Menu;
            }
            KEY_UP => {
                if self.gallery_cursor > 0 {
                    self.gallery_cursor -= 1;
                }
            }
            KEY_DOWN => {
                if self.gallery_cursor < PRESETS.len() - 1 {
                    self.gallery_cursor += 1;
                }
            }
            KEY_ENTER => {
                let preset = PRESETS[self.gallery_cursor];
                self.curve_type = preset.curve;
                self.big_r = preset.big_r;
                self.small_r = preset.small_r;
                self.pen_d = preset.pen_d;
                self.compute_curve();
                self.state = AppState::Drawing;
            }
            _ => {}
        }
        true
    }
}
