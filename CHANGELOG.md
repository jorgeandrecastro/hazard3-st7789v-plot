# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and this project follows [Semantic Versioning](https://semver.org/).

## [0.5.0] — 2026

### Changed
- Migrated to `hazard3-st7789v-plot` and the `hazard3_st7789v_plot` import, in the independent Hazard3 Async project.
- Replaced the legacy Embassy dependencies with `hazard3-st7789v` 0.8 and `hazard3-st7789v-graphics` 0.5; re-exported types come from the new driver.
- The `time-hazard3` backend is enabled by default; `time-upstream` enables upstream Embassy when default features are disabled.
- Legacy `embassy-st7789v-plot` releases were yanked from new dependency resolution.


## [0.4.0] — 2026

### Changed
- No changes to the existing API: fully backward-compatible.
- Re-exported from `embassy-st7789v`: `Color`, `NoPin`, `St7789v`, `St7789vBuffered`, `SCREEN_H`, `SCREEN_W`, `APPROX`, `DEGREE`, `DELTA`, `DIVIDE`, `GE`, `INFINITY`, `LAMBDA`, `LE`, `PI`, `PLUS_MINUS`, `SQRT`, `THETA`, `TIMES`.


## [0.3.0] — 2026

### Added
- `LineChartBuffered<N>`: framebuffer version of `LineChart`, zero flicker
  - `new(config)`: same interface as `LineChart`
  - `push(value)`: identical data insertion
  - `clear()`: reset with `initialized = false`
  - `render_buf(&mut St7789vBuffered)`: full RAM rendering, **synchronous**
  - `draw_f32_buf`: internal helper for drawing floating-point labels in RAM
- Added dependency: `embassy-st7789v-graphics` (for `GraphicsBuffered` and `line_buf`)



## [0.2.0] - 2026-05-29

### Added
- **Two-phase rendering**: Strict separation between static initialization drawing and dynamic grid and curve refreshes to maximize display performance.
- **Automatic zero detection**: Visual highlighting in green (or a configured color) when `0.0` crosses the grid axes.
- **Strict border protection**: Geometric clamping of all drawing primitives to the usable interior to prevent bleeding or overlap with text labels.

### Changed
- Optimized the ring buffer's internal storage to ensure zero stack allocation during asynchronous push operations.

---

## [0.1.1] - 2026-05-27

### Fixed
- **Invalid doc comment in `line()`**: Replaced the `//!` doc comment (reserved for module-level documentation) with a `///` doc comment immediately before the function declaration, in accordance with Rust compiler rules.
- **X label centering**: Corrected the field reference; use `x_axis.label.len()` instead of `y_axis.label.len()` to calculate the centered horizontal position of the X-axis title.
- **X label overflow protection**: Added `.min(SCREEN_H as i32 - 8)` clamping to prevent the X-axis text label from exceeding the physical height of the ST7789V display.