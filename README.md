# Spirograph Engine

A parametric curve visualizer for the [Precursor](https://precursor.dev) handheld, rendering hypotrochoid and epitrochoid spirograph patterns on its 336×536 monochrome 1-bit display.

## Features

- **Two curve types** — Hypotrochoid (rolling inside) and Epitrochoid (rolling outside)
- **Adjustable parameters** — Outer radius R, inner radius r, and pen distance d
- **10 built-in presets** — Classic Star, Pentagon Flower, Cardioid, Nephroid, and more
- **Fixed-point math** — 16.16 integer arithmetic with parabolic sin/cos approximation (no floating-point)
- **Live rendering** — Curve points computed and drawn as connected line segments

## Controls

| Key | Action |
|-----|--------|
| Up/Down | Navigate menu / browse gallery |
| Left/Right | Adjust parameter values |
| Enter | Draw curve / select preset |
| Left/Menu | Return to menu from drawing |
| Menu | Quit from main menu |

## Architecture

- `math.rs` — Fixed-point 16.16 arithmetic, sin/cos via parabolic approximation, hypotrochoid/epitrochoid parametric equations, GCD for curve periodicity
- `app.rs` — 3-state machine (Menu → Drawing → Gallery), 10 presets, curve point computation
- `ui.rs` — Menu rendering, curve drawing via line segments, scrollable gallery
- `main.rs` — Standard Xous event loop

## Presets

| Name | Type | R | r | d |
|------|------|---|---|---|
| Classic Star | Hypotrochoid | 100 | 40 | 30 |
| Pentagon Flower | Hypotrochoid | 100 | 60 | 50 |
| Triangle Rose | Hypotrochoid | 90 | 30 | 25 |
| Seven-Point | Hypotrochoid | 105 | 15 | 12 |
| Cardioid | Epitrochoid | 50 | 50 | 50 |
| Limaçon | Epitrochoid | 60 | 40 | 50 |
| Nephroid | Epitrochoid | 50 | 25 | 25 |
| Astroid | Hypotrochoid | 100 | 25 | 25 |
| Deltoid | Hypotrochoid | 90 | 30 | 30 |
| Spirograph #42 | Hypotrochoid | 96 | 36 | 24 |

---

## Development

This app was developed using the methodology described in [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit) — an LLM-assisted approach to Precursor app development on macOS ARM64.

---

## Author

Made by Tyler Colby — [Colby's Data Movers, LLC](https://colbysdatamovers.com)

Contact: [tyler@colbysdatamovers.com](mailto:tyler@colbysdatamovers.com) | [GitHub Issues](https://github.com/tbcolby/precursor-spirograph/issues)

---

## License

Licensed under the Apache License, Version 2.0.

See [LICENSE](LICENSE) for the full text.
