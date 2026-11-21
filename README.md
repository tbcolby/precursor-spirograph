# precursor-spirograph

Hypotrochoid and epitrochoid curves for [Precursor](https://precursor.dev), drawn as an animated pen on the monochrome display.

## Features

- Curve type (hypotrochoid, rolling inside; epitrochoid, rolling outside), outer radius R (10 to 200), inner radius r (5 up to R-5, or up to R for epitrochoids), pen distance d (5 to 100), all in steps of 5.
- 10 presets: Classic Star, Pentagon Flower, Triangle Rose, Seven-Point, Cardioid, Limaçon, Nephroid, Astroid, Deltoid, Spirograph #42.
- The curve is scaled so its true extent fits the screen (R-r+d for hypotrochoids, R+r+d for epitrochoids) and is traced a few dozen segments per tick. Only new segments are drawn, and the pen stops when the app is in the background or the curve is finished.
- Integer maths only (shared Q16.16 sine table from `precursor-appkit-core`).

## How the curve is computed

With `g = gcd(R, r)`, the curve closes after `r/g` turns of the rolling circle. The app parametrises one full closed curve over a single turn `u` and uses whole-number multiples of `u` as the angles (`r/g` and `R/g -/+ r/g`), so there is no fixed-point ratio to truncate and every curve is complete (an earlier version capped the turns and left 179 of 780 combinations open). Points are computed from the index on demand; segment count is between 256 and 6144 and grows with the number of lobes so segments stay short. The scale and pen sizes are computed once in Q16.16 and shifted to pixels at the end.

## Keys

| Screen | Keys |
|---|---|
| Menu | Up/Down field, Left/Right change a value (Enter or Left/Right toggles the type), Enter on Draw or Gallery, Backspace or F4 quit |
| Drawing | Enter restart the pen, Backspace or F4 back |
| Gallery | Up/Down, Enter draws, Backspace or F4 back |

Lowering R re-clamps r so the menu never shows an impossible state. The menu key (centre of the D-pad) is consumed by Xous and never reaches the app.

## Storage

None. Nothing is saved.

## Limitations

- Layout has not been checked on a device. Renode capture not yet run: `capture.py` is written but not executed; verification so far is the hosted build, clippy, lint and host tests.
- Curves with very many lobes (R and r nearly equal, small gcd) are sampled at the 6144-segment cap and take a few seconds to trace.
- An epitrochoid's drawn size is scaled by the analytic extent bound, which is exact only where the pen aligns; some epitrochoids fill slightly less than the full box.

## Building

```bash
bin/integrate --xous-root ~/xous-core --grade ~/Dev/precursor/precursor-spirograph
```

Host tests alone: `cd core && cargo test`.

## Layout

```
core/src/lib.rs     params, presets, Curve (on-demand points), Pen (incremental)
core/src/tests.rs   f64 reference, closure over the whole menu grid, extent, long periods
src/main.rs         GAM registration, pump control (foreground and animating only)
src/app.rs          key mapping and drawing
capture.py          Renode capture sequence (not yet run)
```

## Author

Tyler Colby, [Colby's Data Movers, LLC](https://colbysdatamovers.com). Licensed under the Apache License, Version 2.0 (see [LICENSE](LICENSE)).
