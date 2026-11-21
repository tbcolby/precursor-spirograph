//! Pure spirograph maths: hypotrochoids and epitrochoids in integer
//! arithmetic using the shared Q16.16 trig, the preset table, and an
//! incremental pen. No Xous dependencies; runs under `cargo test`.
//!
//! The curve is parameterised by `u` over one full turn so that it closes
//! exactly, whatever the radii. With `g = gcd(R, r)`, `R' = R/g`, `r' = r/g`:
//!
//! ```text
//! hypo:  x = (R-r) cos(r' u) + d cos((R'-r') u)    y = (R-r) sin(r' u) - d sin((R'-r') u)
//! epi:   x = (R+r) cos(r' u) - d cos((R'+r') u)    y = (R+r) sin(r' u) - d sin((R'+r') u)
//! ```
//!
//! Angles are whole multiples of `u`, so the old fixed-point ratio
//! `(R-r)/r` (and its truncation) never appears. Everything is scaled in
//! Q16.16 and shifted down to pixels once, at the end.

pub use precursor_appkit_core::fixed;
use precursor_appkit_core::fixed::{cos, mul, sin, step_angle};

pub const BIG_R_MIN: i32 = 10;
pub const BIG_R_MAX: i32 = 200;
pub const SMALL_R_MIN: i32 = 5;
pub const PEN_D_MIN: i32 = 5;
pub const PEN_D_MAX: i32 = 100;
pub const STEP: i32 = 5;
/// Upper bound on segments per curve (memory is constant: points are computed on demand).
pub const MAX_SEGMENTS: i32 = 6144;
pub const MIN_SEGMENTS: i32 = 256;
/// Segments revealed per animation tick.
pub const SEGMENTS_PER_TICK: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Hypotrochoid,
    Epitrochoid,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Hypotrochoid => "Hypotrochoid",
            Kind::Epitrochoid => "Epitrochoid",
        }
    }

    pub fn toggle(self) -> Kind {
        match self {
            Kind::Hypotrochoid => Kind::Epitrochoid,
            Kind::Epitrochoid => Kind::Hypotrochoid,
        }
    }
}

/// Curve parameters; always satisfies `SMALL_R_MIN <= r <= R - STEP` for a
/// hypotrochoid (the small circle must fit inside) and `r <= R` for an
/// epitrochoid (it rolls outside, and `r == R` is the Cardioid).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Params {
    pub kind: Kind,
    pub big_r: i32,
    pub small_r: i32,
    pub pen_d: i32,
}

impl Params {
    pub fn new(kind: Kind, big_r: i32, small_r: i32, pen_d: i32) -> Self {
        let big_r = big_r.clamp(BIG_R_MIN, BIG_R_MAX);
        let max_r = if kind == Kind::Hypotrochoid { big_r - STEP } else { big_r };
        Self {
            kind,
            big_r,
            small_r: small_r.clamp(SMALL_R_MIN, max_r),
            pen_d: pen_d.clamp(PEN_D_MIN, PEN_D_MAX),
        }
    }

    /// Change R by `delta` steps; r is re-clamped (audit: it used to be left
    /// at or above R after lowering R).
    pub fn step_big_r(&mut self, delta: i32) { *self = Self::new(self.kind, self.big_r + delta * STEP, self.small_r, self.pen_d); }

    pub fn step_small_r(&mut self, delta: i32) { *self = Self::new(self.kind, self.big_r, self.small_r + delta * STEP, self.pen_d); }

    /// Switch between hypotrochoid and epitrochoid, re-clamping r.
    pub fn toggle_kind(&mut self) { *self = Self::new(self.kind.toggle(), self.big_r, self.small_r, self.pen_d); }

    pub fn step_pen_d(&mut self, delta: i32) { *self = Self::new(self.kind, self.big_r, self.small_r, self.pen_d + delta * STEP); }

    /// Full turns of the rolling circle before the curve closes: `r / gcd(R, r)`.
    pub fn period_turns(&self) -> i32 { self.small_r / gcd(self.big_r, self.small_r) }

    /// Largest distance from the centre the pen can reach (an upper bound;
    /// exact for hypotrochoids and for epitrochoids where the pen aligns).
    pub fn extent(&self) -> i32 {
        match self.kind {
            Kind::Hypotrochoid => (self.big_r - self.small_r) + self.pen_d,
            Kind::Epitrochoid => self.big_r + self.small_r + self.pen_d,
        }
    }
}

pub fn gcd(a: i32, b: i32) -> i32 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

#[derive(Debug, Clone, Copy)]
pub struct Preset {
    pub name: &'static str,
    pub params: Params,
}

const fn preset(name: &'static str, kind: Kind, big_r: i32, small_r: i32, pen_d: i32) -> Preset {
    Preset { name, params: Params { kind, big_r, small_r, pen_d } }
}

pub const PRESETS: &[Preset] = &[
    preset("Classic Star", Kind::Hypotrochoid, 100, 40, 30),
    preset("Pentagon Flower", Kind::Hypotrochoid, 100, 60, 50),
    preset("Triangle Rose", Kind::Hypotrochoid, 90, 30, 25),
    preset("Seven-Point", Kind::Hypotrochoid, 105, 15, 12),
    preset("Cardioid", Kind::Epitrochoid, 50, 50, 50),
    preset("Limaçon", Kind::Epitrochoid, 60, 40, 50),
    preset("Nephroid", Kind::Epitrochoid, 50, 25, 25),
    preset("Astroid", Kind::Hypotrochoid, 100, 25, 25),
    preset("Deltoid", Kind::Hypotrochoid, 90, 30, 30),
    preset("Spirograph #42", Kind::Hypotrochoid, 96, 36, 24),
];

/// A sampled spirograph: `len()` segments between `len() + 1` points,
/// computed on demand.
#[derive(Debug, Clone, Copy)]
pub struct Curve {
    kind: Kind,
    /// r / gcd: the inner angle is `rp * u`.
    rp: i32,
    /// (R' -/+ r'): the pen angle is `f2 * u`.
    f2: i32,
    n: i32,
    center: (i32, i32),
    /// Pixel size of the two terms in Q16.16.
    a_q: i32,
    d_q: i32,
}

impl Curve {
    /// Curve scaled so its extent is `fit` pixels, centred at `center`.
    pub fn new(p: Params, center: (i32, i32), fit: i32) -> Self {
        let g = gcd(p.big_r, p.small_r);
        let (rp, bp) = (p.small_r / g, p.big_r / g);
        let (f2, k) = match p.kind {
            Kind::Hypotrochoid => (bp - rp, p.big_r - p.small_r),
            Kind::Epitrochoid => (bp + rp, p.big_r + p.small_r),
        };
        let extent = p.extent().max(1);
        // One 64-bit divide per curve (not per point); pixels per unit in Q16.16.
        let scale = (((fit.max(1) as i64) << 16) / extent as i64) as i32;
        let n = (64 * rp.max(f2.abs())).clamp(MIN_SEGMENTS, MAX_SEGMENTS);
        Self { kind: p.kind, rp, f2, n, center, a_q: k * scale, d_q: p.pen_d * scale }
    }

    pub fn len(&self) -> usize { self.n as usize }

    pub fn is_empty(&self) -> bool { false }

    /// Point `i` for `i` in `0..=len()`; `point(len()) == point(0)`.
    pub fn point(&self, i: usize) -> (i32, i32) {
        let i = i as i32;
        let a1 = step_angle(self.rp * i, self.n);
        let a2 = step_angle(self.f2 * i, self.n);
        let (dx, dy) = (mul(self.d_q, cos(a2)), mul(self.d_q, sin(a2)));
        let x = mul(self.a_q, cos(a1)) + if self.kind == Kind::Hypotrochoid { dx } else { -dx };
        let y = mul(self.a_q, sin(a1)) - dy;
        (self.center.0 + ((x + (1 << 15)) >> 16), self.center.1 + ((y + (1 << 15)) >> 16))
    }

    pub fn segments(&self, range: core::ops::Range<usize>) -> impl Iterator<Item = ((i32, i32), (i32, i32))> + '_ {
        let end = range.end.min(self.len());
        (range.start..end).map(move |i| (self.point(i), self.point(i + 1)))
    }
}

/// Reveals a curve a few segments at a time (the animated pen). Each
/// segment index is returned exactly once.
#[derive(Debug, Clone, Copy)]
pub struct Pen {
    pub curve: Curve,
    drawn: usize,
}

impl Pen {
    pub fn new(curve: Curve) -> Self { Self { curve, drawn: 0 } }

    pub fn drawn(&self) -> usize { self.drawn }

    pub fn done(&self) -> bool { self.drawn >= self.curve.len() }

    pub fn advance(&mut self, n: usize) -> core::ops::Range<usize> {
        let start = self.drawn;
        self.drawn = (self.drawn + n).min(self.curve.len());
        start..self.drawn
    }
}

#[cfg(test)]
mod tests;
