use super::*;
use precursor_appkit_core::fixed::TURN;

const C: (i32, i32) = (168, 200);

fn grid() -> impl Iterator<Item = (i32, i32)> {
    (BIG_R_MIN..=BIG_R_MAX).step_by(5).flat_map(|r| (SMALL_R_MIN..=r - STEP).step_by(5).map(move |s| (r, s)))
}

fn reference(p: &Params, c: &Curve, fit: i32, i: usize) -> (f64, f64) {
    let (r, s, d) = (p.big_r as f64, p.small_r as f64, p.pen_d as f64);
    let t = core::f64::consts::TAU * p.period_turns() as f64 * i as f64 / c.len() as f64;
    let scale = fit as f64 / p.extent() as f64;
    let (x, y) = match p.kind {
        Kind::Hypotrochoid => (
            (r - s) * t.cos() + d * ((r - s) / s * t).cos(),
            (r - s) * t.sin() - d * ((r - s) / s * t).sin(),
        ),
        Kind::Epitrochoid => (
            (r + s) * t.cos() - d * ((r + s) / s * t).cos(),
            (r + s) * t.sin() - d * ((r + s) / s * t).sin(),
        ),
    };
    (C.0 as f64 + x * scale, C.1 as f64 + y * scale)
}

#[test]
fn audit_type_error_regression_fixed_point_path() {
    // The old fp_div returned i64 from an i32 fn (crate did not compile), and the
    // app passed plain integers where fixed point was expected. Presets must match
    // an f64 reference to a pixel.
    for pr in PRESETS {
        let c = Curve::new(pr.params, C, 140);
        for i in (0..=c.len()).step_by(5) {
            let (x, y) = reference(&pr.params, &c, 140, i);
            let (px, py) = c.point(i);
            assert!(
                (px as f64 - x).abs() <= 1.5 && (py as f64 - y).abs() <= 1.5,
                "{} i={i}: ({px},{py}) vs ({x:.1},{y:.1})",
                pr.name
            );
        }
    }
}

#[test]
fn closes_for_every_reachable_radius_triple() {
    for (r, s) in grid() {
        for kind in [Kind::Hypotrochoid, Kind::Epitrochoid] {
            for d in [PEN_D_MIN, 30, PEN_D_MAX] {
                let c = Curve::new(Params::new(kind, r, s, d), C, 140);
                assert_eq!(c.point(0), c.point(c.len()), "{kind:?} {r}/{s}/{d}");
            }
        }
    }
}

#[test]
fn audit_long_period_curves_are_complete() {
    // R=200, r=195 needs 39 turns; the old 16-turn cap left it open. Here the
    // whole 39-turn curve is sampled within bounds and matches the reference.
    let p = Params::new(Kind::Hypotrochoid, 200, 195, 50);
    assert_eq!(p.period_turns(), 39);
    let c = Curve::new(p, C, 140);
    assert!(c.len() as i32 <= MAX_SEGMENTS);
    for i in (0..=c.len()).step_by(11) {
        let (x, y) = reference(&p, &c, 140, i);
        let (px, py) = c.point(i);
        assert!((px as f64 - x).abs() <= 1.5 && (py as f64 - y).abs() <= 1.5, "i={i}");
    }
}

#[test]
fn period_turns_matches_gcd_rule() {
    assert_eq!(Params::new(Kind::Hypotrochoid, 100, 40, 30).period_turns(), 2);
    assert_eq!(Params::new(Kind::Hypotrochoid, 105, 15, 12).period_turns(), 1);
    assert_eq!(Params::new(Kind::Epitrochoid, 50, 50, 50).period_turns(), 1);
    assert_eq!(gcd(0, 0), 1);
    assert_eq!(gcd(-12, 18), 6);
}

#[test]
fn audit_extent_hypo_uses_r_minus_r_and_epi_fits() {
    // Hypotrochoid reaches exactly (R - r + d); the old code scaled by R + d and drew small.
    let p = Params::new(Kind::Hypotrochoid, 100, 40, 30);
    assert_eq!(p.extent(), 90);
    let c = Curve::new(p, C, 140);
    let reach = (0..=c.len()).map(|i| dist(c.point(i))).fold(0.0f64, f64::max);
    assert!((reach - 140.0).abs() <= 2.0, "reach {reach}");
    // Epitrochoids (old code overflowed: Cardioid reached 210 px) stay inside `fit`.
    for pr in PRESETS {
        let c = Curve::new(pr.params, C, 140);
        for i in 0..=c.len() {
            assert!(dist(c.point(i)) <= 141.5, "{} reaches {}", pr.name, dist(c.point(i)));
        }
    }
    let card = Curve::new(PRESETS[4].params, C, 140);
    let reach = (0..=card.len()).map(|i| dist(card.point(i))).fold(0.0f64, f64::max);
    assert!((reach - 140.0).abs() <= 2.0, "cardioid reach {reach}");
}

fn dist(p: (i32, i32)) -> f64 { (((p.0 - C.0).pow(2) + (p.1 - C.1).pow(2)) as f64).sqrt() }

#[test]
fn every_reachable_curve_fits_and_stays_smooth() {
    for (r, s) in grid() {
        for kind in [Kind::Hypotrochoid, Kind::Epitrochoid] {
            for d in [PEN_D_MIN, PEN_D_MAX] {
                let p = Params::new(kind, r, s, d);
                let c = Curve::new(p, C, 140);
                assert!((MIN_SEGMENTS..=MAX_SEGMENTS).contains(&(c.len() as i32)));
                for (a, b) in c.segments(0..c.len()) {
                    assert!(dist(a) <= 141.5, "{p:?} out of fit");
                    let seg = (a.0 - b.0).abs().max((a.1 - b.1).abs());
                    assert!(seg <= 16, "{p:?}: {seg}px segment");
                }
            }
        }
    }
}

#[test]
fn audit_lowering_big_r_reclamps_small_r() {
    let mut p = Params::new(Kind::Hypotrochoid, 100, 40, 30);
    for _ in 0..14 {
        p.step_big_r(-1);
        assert!(p.small_r <= p.big_r - STEP && p.small_r >= SMALL_R_MIN, "{p:?}");
    }
    assert_eq!(p.big_r, 30);
    assert_eq!(p.small_r, 25);
    p.step_big_r(-100);
    assert_eq!(p.big_r, BIG_R_MIN);
    assert_eq!(p.small_r, 5);
    p.step_small_r(5);
    assert_eq!(p.small_r, 5);
    // Epitrochoid may reach r == R (Cardioid); switching back to hypo re-clamps.
    let mut e = Params::new(Kind::Epitrochoid, 50, 50, 50);
    assert_eq!(e.small_r, 50);
    e.toggle_kind();
    assert_eq!((e.kind, e.small_r), (Kind::Hypotrochoid, 45));
    p.step_pen_d(100);
    assert_eq!(p.pen_d, PEN_D_MAX);
}

#[test]
fn presets_have_their_named_shapes() {
    // Cusps sit at the full extent: count runs of samples within 3% of it.
    let cusps = |name: &str| {
        let pr = PRESETS.iter().find(|p| p.name == name).unwrap();
        let c = Curve::new(pr.params, C, 140);
        let n = c.len();
        let hot: Vec<bool> = (0..n).map(|i| dist(c.point(i)) > 136.0).collect();
        (0..n).filter(|&i| hot[i] && !hot[(i + n - 1) % n]).count()
    };
    assert_eq!(cusps("Deltoid"), 3);
    assert_eq!(cusps("Astroid"), 4);
    assert_eq!(cusps("Cardioid"), 1);
    let neph = PRESETS.iter().find(|p| p.name == "Nephroid").unwrap().params;
    assert_eq!((neph.big_r, neph.small_r, neph.pen_d), (50, 25, 25));
    assert_eq!(cusps("Nephroid"), 2);
}

#[test]
fn pen_yields_each_segment_once() {
    let c = Curve::new(PRESETS[0].params, C, 100);
    let mut pen = Pen::new(c);
    let mut seen = vec![false; c.len()];
    while !pen.done() {
        for i in pen.advance(SEGMENTS_PER_TICK) {
            assert!(!seen[i]);
            seen[i] = true;
        }
    }
    assert!(seen.iter().all(|&s| s));
    assert!(pen.advance(5).is_empty());
}

#[test]
fn symmetric_under_one_symmetry_rotation() {
    // Classic Star (R'=5, r'=2): the figure has 5-fold symmetry (R' = 5 lobes).
    let p = Params::new(Kind::Hypotrochoid, 100, 40, 30);
    let c = Curve::new(p, C, 140);
    let n = c.len();
    let pts: Vec<(i32, i32)> = (0..n).map(|i| c.point(i)).collect();
    for i in (0..n).step_by(13) {
        let q = fixed::rotate(c.point(i), C, TURN / 5);
        // Samples are a few px apart, so allow the polyline's chord length.
        let near = pts.iter().any(|p| (p.0 - q.0).abs() <= 6 && (p.1 - q.1).abs() <= 6);
        assert!(near, "rotated point {q:?} not on curve");
    }
}

#[test]
fn preset_params_are_valid() {
    for pr in PRESETS {
        let p = pr.params;
        assert_eq!(p, Params::new(p.kind, p.big_r, p.small_r, p.pen_d), "{}", pr.name);
    }
    assert_eq!(Kind::Hypotrochoid.toggle().toggle(), Kind::Hypotrochoid);
    assert_eq!(Kind::Epitrochoid.label(), "Epitrochoid");
}
