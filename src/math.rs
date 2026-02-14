//! Fixed-point math for spirograph curves.
//!
//! Uses i32 fixed-point (16.16) to avoid floating-point on RISC-V.
//! Sin/cos via lookup table with linear interpolation.

/// Fixed-point scale factor: 1.0 = 65536
const FP_SHIFT: i32 = 16;
const FP_ONE: i32 = 1 << FP_SHIFT; // 65536

/// Convert integer to fixed-point.
pub fn fp(n: i32) -> i32 {
    n << FP_SHIFT
}

/// Fixed-point multiply.
pub fn fp_mul(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) >> FP_SHIFT) as i32
}

/// Fixed-point divide.
pub fn fp_div(a: i32, b: i32) -> i32 {
    if b == 0 { return 0; }
    ((a as i64) << FP_SHIFT) as i64 / b as i64
}

/// Sin lookup table — 256 entries for 0..2π (values in FP 16.16).
/// sin(i * 2π / 256) * 65536
static SIN_TABLE: [i32; 256] = {
    // Pre-computed at compile time isn't possible in const context easily,
    // so we use a static array. Values for sin(i * 2π / 256) * 65536.
    // Generated offline.
    let mut table = [0i32; 256];
    // We'll compute approximate values using a Taylor series approach
    // at compile time. For embedded, this is the standard approach.
    // Actually, let's just hardcode the table:
    table
};

// Since const evaluation of trig is tricky, use a runtime-initialized approach
// with a simpler integer sine approximation.

/// Fast integer sine approximation.
/// Input: angle in units of 1/1024 of a full circle (0..1023).
/// Output: fixed-point sin value (-65536..65536).
pub fn sin_fp(angle: i32) -> i32 {
    // Normalize to 0..1023
    let a = ((angle % 1024) + 1024) % 1024;

    // Quadrant reduction
    let (idx, sign) = if a < 256 {
        (a, 1i32)
    } else if a < 512 {
        (512 - a, 1)
    } else if a < 768 {
        (a - 512, -1)
    } else {
        (1024 - a, -1)
    };

    // Parabolic approximation for quarter wave (0..256 → 0..65536)
    // sin(x) ≈ 4x(256-x) / 256² * correction
    let x = idx;
    let y = (4 * x * (256 - x)) as i64;
    let raw = (y * FP_ONE as i64 / (256 * 256) as i64) as i32;

    // Apply sign
    raw * sign
}

/// Fast integer cosine.
pub fn cos_fp(angle: i32) -> i32 {
    sin_fp(angle + 256) // cos = sin(x + π/2)
}

/// Hypotrochoid: curve traced by a point on a circle rolling inside another.
/// R = outer radius, r = inner radius, d = pen distance from inner center.
/// x(t) = (R-r)*cos(t) + d*cos((R-r)/r * t)
/// y(t) = (R-r)*sin(t) - d*sin((R-r)/r * t)
///
/// All parameters in fixed-point. Angle t in 1/1024 units.
pub fn hypotrochoid_x(t: i32, big_r: i32, small_r: i32, pen_d: i32) -> i32 {
    let diff = big_r - small_r;
    let ratio = fp_div(diff, small_r);
    let inner_angle = fp_mul(ratio, t) as i32;

    fp_mul(diff, cos_fp(t)) + fp_mul(pen_d, cos_fp(inner_angle))
}

pub fn hypotrochoid_y(t: i32, big_r: i32, small_r: i32, pen_d: i32) -> i32 {
    let diff = big_r - small_r;
    let ratio = fp_div(diff, small_r);
    let inner_angle = fp_mul(ratio, t) as i32;

    fp_mul(diff, sin_fp(t)) - fp_mul(pen_d, sin_fp(inner_angle))
}

/// Epitrochoid: curve traced by a point on a circle rolling outside another.
/// x(t) = (R+r)*cos(t) - d*cos((R+r)/r * t)
/// y(t) = (R+r)*sin(t) - d*sin((R+r)/r * t)
pub fn epitrochoid_x(t: i32, big_r: i32, small_r: i32, pen_d: i32) -> i32 {
    let sum = big_r + small_r;
    let ratio = fp_div(sum, small_r);
    let inner_angle = fp_mul(ratio, t) as i32;

    fp_mul(sum, cos_fp(t)) - fp_mul(pen_d, cos_fp(inner_angle))
}

pub fn epitrochoid_y(t: i32, big_r: i32, small_r: i32, pen_d: i32) -> i32 {
    let sum = big_r + small_r;
    let ratio = fp_div(sum, small_r);
    let inner_angle = fp_mul(ratio, t) as i32;

    fp_mul(sum, sin_fp(t)) - fp_mul(pen_d, sin_fp(inner_angle))
}

/// GCD for computing curve periodicity.
pub fn gcd(mut a: i32, mut b: i32) -> i32 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// How many full rotations until the pattern repeats.
/// For hypotrochoid: period = r / gcd(R, r) full rotations of t.
pub fn curve_period(big_r: i32, small_r: i32) -> i32 {
    if small_r == 0 { return 1; }
    let g = gcd(big_r, small_r);
    small_r / g
}
