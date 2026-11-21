# Spirograph build notes

- `core/src/lib.rs`: one closed curve over `u` in one turn, angles `r' u` and `(R' -/+ r') u` with `R'=R/g`, `r'=r/g`. Scale and term sizes in Q16.16, one shift at the end. No `fp_div` per point.
- Params invariant: hypo `r <= R-5`, epi `r <= R` (Cardioid); `step_big_r` and `toggle_kind` re-clamp.
- Points computed from the index; `Pen::advance` yields each segment once; the shell draws only new segments per tick.
- Pump runs only when foreground and `App::animating()`.
- Tests are heavy-ish (the menu-grid test is ~20 s in debug): `cd core && cargo test`.
