# hazard3-st7789v-plot

A configurable Cartesian (X, Y) plotting engine for **ST7789V 240×320** TFT LCD displays, built on [`hazard3-st7789v`](https://crates.io/crates/hazard3-st7789v).
It is part of [Hazard3 Async](https://github.com/jorgeandrecastro/hazard3-async), an independent ecosystem for the RP2350's RISC-V Hazard3 cores, not affiliated with the Embassy project.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20or%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
[![docs.rs](https://docs.rs/hazard3-st7789v-plot/badge.svg)](https://docs.rs/hazard3-st7789v-plot)


## Features

- **`#![no_std]` + `#![forbid(unsafe_code)]`**: Safe and embedded-friendly
- **No dynamic allocation**: Fixed-size ring buffer
- **Circular history**: Up to 240 points (the screen's physical width)
- **Configurable axes**: Static ticks with configurable spacing
- **Configurable grid**: Horizontal and vertical grid with custom labels
- **Automatic zero detection**: The zero line is highlighted in green
- **Strict border protection**: Drawing primitives are clamped to the interior
- **Bresenham line rendering**: Smooth curves between data points
- **Two-phase rendering**: Frame, labels, and titles are drawn only once
- **Two rendering modes**:
  - `LineChart` + `render()`: async, direct SPI
  - `LineChartBuffered` + `render_buf()` + `flush()`: synchronous RAM rendering, reducing flicker without guaranteeing tear-free updates

---

## Installation

```toml
[dependencies]
hazard3-st7789v-plot = "0.6"
```

Version 0.6 of this new package must be published on crates.io before it can be installed.
The default `time-hazard3` backend uses `hazard3-time`: the application must provide a compatible time driver and runtime. See the [Hazard3 Async quick start](https://github.com/jorgeandrecastro/hazard3-async#quick-start).

To use the official Embassy backend instead:

```toml
[dependencies]
hazard3-st7789v-plot = { version = "0.6", default-features = false, features = ["time-upstream"] }
```

Enable exactly one backend and use a compatible time driver.
If the driver or graphics layer are also direct dependencies, use
`hazard3-st7789v = "0.9"` and `hazard3-st7789v-graphics = "0.6"` with the same
backend. For `time-upstream`, disable their default features as well.
Cargo features are additive: do not mix the two backends.

### Migration from `embassy-st7789v-plot`

Replace the package and import `embassy_st7789v_plot` with
`hazard3-st7789v-plot` and `hazard3_st7789v_plot`. Also replace the
`embassy-st7789v` and `embassy-st7789v-graphics` dependencies with their
`hazard3-*` counterparts: types from the old driver are not interchangeable
with those from the new one. Yanked releases remain usable through an existing
`Cargo.lock`.

This crate also re-exports items from `hazard3-st7789v`, including `Color`, `NoPin`, `St7789v`, `St7789vBuffered`, `SCREEN_H`, `SCREEN_W`, `APPROX`, `DEGREE`, `DELTA`, `DIVIDE`, `GE`, `INFINITY`, `LAMBDA`, `LE`, `PI`, `PLUS_MINUS`, `SQRT`, `THETA`, and `TIMES`.

### Example: text with symbols

```rust
use hazard3_st7789v_plot::{Color, St7789v, St7789vBuffered, SCREEN_H, DEGREE, INFINITY, PI};

let mut display = St7789vBuffered::new(St7789v::new(spi_dev, dc, rst));
let bg = Color::BLACK;
let H = SCREEN_H as u16;

// ---- TEXT WITH SYMBOLS ----
let _ = display.draw_str_scaled_buf(20, 2, b"GRAPHICS DEMO", Color::CYAN, bg, 2);
let _ = display.draw_str_scaled_buf(2, H as u16 - 48, b"Line  Circle  Triangle  Ellipse  Bezier", Color::WHITE, bg, 1);

// Display ∞
let _ = display.draw_str_scaled_buf(2, H as u16 - 32, b"INFINITY ", Color::WHITE, bg, 1);
let _ = display.draw_str_scaled_buf(2 + 9 * 6, H as u16 - 32, &[INFINITY], Color::YELLOW, bg, 1);

// Display π
let _ = display.draw_str_scaled_buf(2, H as u16 - 20, b"PI ", Color::WHITE, bg, 1);
let _ = display.draw_str_scaled_buf(2 + 3 * 6, H as u16 - 20, &[PI], Color::YELLOW, bg, 1);

// Display °
let _ = display.draw_str_scaled_buf(2 + 4 * 6 + 6, H as u16 - 20, b"DEG ", Color::WHITE, bg, 1);
let _ = display.draw_str_scaled_buf(2 + 4 * 6 + 6 + 4 * 6, H as u16 - 20, &[DEGREE], Color::YELLOW, bg, 1);
```

---

## Quick start

### Async direct mode (`LineChart`)

Simple, but may flicker if the chart is redrawn frequently.

```rust
use hazard3_st7789v_plot::{Color, Graphics, AxisConfig, PlotConfig, LineChart};

let x_axis = AxisConfig::new(0.0, 10.0, 2.0, b"Time (s)");
let y_axis = AxisConfig::new(0.0, 100.0, 20.0, b"Temperature (C)");

let config = PlotConfig {
    x: 10, y: 10, width: 220, height: 200,
    margin_left: 40, margin_right: 10,
    margin_top: 10, margin_bottom: 30,
    x_axis, y_axis,
    bg_color:   Color::BLACK,
    line_color: Color::GREEN,
    axis_color: Color::WHITE,
    grid_color: Color::GRAY,
    text_color: Color::WHITE,
    label_color: Color::CYAN,
};

let mut chart: LineChart<100> = LineChart::new(config);
chart.push(45.2);
chart.push(47.8);

let mut gfx = Graphics::new_no_rst(&mut display);
chart.render(&mut gfx).await;
```

### Synchronous framebuffer mode (`LineChartBuffered`)

All drawing operations write to RAM. A single `flush()` sends the frame to the display.
Requires `St7789vBuffered` from `hazard3-st7789v` (150 KiB of RAM).

```rust
use hazard3_st7789v_plot::{Color, St7789v, St7789vBuffered, AxisConfig, PlotConfig, LineChartBuffered};

let mut display = St7789vBuffered::new(St7789v::new(spi_dev, dc, rst));
display.driver().init().await.unwrap();

let config = PlotConfig { /* ... */ };
let mut chart: LineChartBuffered<100> = LineChartBuffered::new(config);

loop {
    let value = sensor.read().await;
    chart.push(value);

    chart.render_buf(&mut display);   // synchronous RAM rendering
    display.flush().await.unwrap();   // send the framebuffer over SPI
}
```

---

## Axis configuration

```rust
// Time 0–60 s, ticks every 10 s
let time_axis = AxisConfig::new(0.0, 60.0, 10.0, b"Time (s)");

// Temperature -10 to +50 °C, ticks every 10 °C
let temp_axis = AxisConfig::new(-10.0, 50.0, 10.0, b"Temp (C)");

// Voltage 0–3.3 V, ticks every 0.5 V
let volt_axis = AxisConfig::new(0.0, 3.3, 0.5, b"U (V)");

// Number of ticks
assert_eq!(time_axis.tick_count(), 7); // 0, 10, 20, 30, 40, 50, 60
```

---

## Layout diagram
(x, y) ┌──────────────────────────────────┐
│          margin_top                     │
│  ┌──────────────────────────┐           │
│  │ Label Y                  │           │
margin │  │  50 ┼──────────•        │    │ margin
left  │  │  25 │      •       •     │   │ right
│  │   0 └──────────────────  │   │
│  │     0   2   4   6   8 10 │   │
│  │          Label X         │   │
│  └──────────────────────────┘   │
│          margin_bottom          │
└──────────────────────────────────┘

---

## Rendering workflow (two phases)

**Initialization phase (once):**
1. Clear the display area
2. Y labels — tick values (green when zero is detected)
3. X labels — tick values
4. Axis titles
5. Fixed borders (axis_color)

**Dynamic phase (on each render):**
1. Clear the interior without touching the borders
2. Y grid — horizontal lines (green for zero)
3. X grid — vertical lines
4. Curve — Bresenham lines connecting the points

---

## Ring buffer (circular history)
array[100], head=3, count=100 (full)
data[0] = 97th value
data[1] = 98th value
data[2] = 99th value ← head (next write)
data[3] = 1st value ← oldest
...

---

## Use cases

### Single-channel oscilloscope

```rust
let config = PlotConfig {
    x: 0, y: 0, width: 240, height: 320,
    margin_left: 40, margin_right: 10,
    margin_top: 10, margin_bottom: 30,
    x_axis: AxisConfig::new(0.0, 240.0, 30.0, b"Samples"),
    y_axis: AxisConfig::new(-5.0, 5.0, 1.0, b"Voltage (V)"),
    // ...
};
let mut osc: LineChartBuffered<240> = LineChartBuffered::new(config);
```

### Temperature monitor

```rust
let config = PlotConfig {
    x: 10, y: 10, width: 220, height: 150,
    margin_left: 45, margin_right: 15,
    margin_top: 15, margin_bottom: 30,
    x_axis: AxisConfig::new(0.0, 120.0, 20.0, b"Time (min)"),
    y_axis: AxisConfig::new(15.0, 35.0, 5.0, b"T (C)"),
    // ...
};
let mut temp: LineChartBuffered<120> = LineChartBuffered::new(config);
```

### Pressure chart

```rust
let config = PlotConfig {
    x: 5, y: 5, width: 230, height: 310,
    margin_left: 35, margin_right: 5,
    margin_top: 5, margin_bottom: 30,
    x_axis: AxisConfig::new(0.0, 1000.0, 100.0, b"Pa"),
    y_axis: AxisConfig::new(900.0, 1050.0, 30.0, b"P (hPa)"),
    // ...
};
let mut pres: LineChartBuffered<100> = LineChartBuffered::new(config);
```

---

## Complete API

### `LineChart<N>` and `LineChartBuffered<N>`

| Method | `LineChart` | `LineChartBuffered` |
|---|---|---|
| `new(config)` | ✓ | ✓ |
| `push(value)` | ✓ | ✓ |
| `clear()` | ✓ | ✓ |
| `config()` | ✓ | — |
| `render(&mut gfx)` | ✓ async | — |
| `render_buf(&mut display)` | — | ✓ sync |

### `AxisConfig`

| Method | Description |
|---|---|
| `new(start, end, step, label)` | Creates an axis configuration |
| `is_valid()` | Checks validity |
| `tick_count()` | Number of ticks |

### `Graphics<'a, SPI, DC, RST>`

| Method | Description |
|---|---|
| `new(display)` | Creates a context with RST |
| `new_no_rst(display)` | Creates a context without RST |
| `pixel(x, y, color)` | Draws a pixel (async) |

### Global functions

| Function | Mode | Description |
|---|---|---|
| `line(gfx, x0, y0, x1, y1, color)` | async | Bresenham direct SPI |

---

## Limitations

- **Number of points**: Maximum 240 (physical screen width)
- **Precision**: Floating-point axes, integer pixels
- **Non-adaptive step**: Static ticks, no automatic zoom
- **Framebuffer RAM**: `St7789vBuffered` requires 150 KiB; ensure sufficient RAM and stack space.

---

## 🛡️ License

Licensed under either of:

    MIT license (http://opensource.org/licenses/MIT)

    Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## 🦅 About

Developed by Jorge Andre Castro.

---

## Dependencies

- [`hazard3-st7789v`](https://crates.io/crates/hazard3-st7789v) — ST7789V driver + framebuffer
- [`hazard3-st7789v-graphics`](https://crates.io/crates/hazard3-st7789v-graphics) — Geometric primitives
- [`embedded-trig-f32`](https://crates.io/crates/embedded-trig-f32) — Trigonometric functions
- [`embedded-hal`](https://crates.io/crates/embedded-hal) — HAL traits
- [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async) — Async HAL traits