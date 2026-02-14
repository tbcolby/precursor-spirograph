# Spirograph Engine — Build Notes

## What This Is
Parametric curve visualizer for Precursor. Renders hypotrochoid and epitrochoid spirograph patterns on the 1-bit display using fixed-point integer math.

## Key Decisions
- **No floating-point**: All trig via 16.16 fixed-point with parabolic sin/cos approximation (1024 units per revolution)
- **No PDDB/storage**: Pure visual app, no saved state needed
- **No TRNG**: Deterministic curves, no randomness
- **Precomputed points**: Curve is fully computed into Vec<(i16, i16)> before transitioning to Drawing state, avoiding per-frame computation
- **Line segment rendering**: Points drawn as connected lines via gam.draw_line(), clipped to content area
- **Capped iterations**: max 16,384 points per curve to prevent excessive computation

## Architecture
- 3 states: Menu, Drawing, Gallery
- 10 presets covering both curve types
- Fixed-point math module is self-contained and reusable for other visual apps (Lissajous, etc.)
