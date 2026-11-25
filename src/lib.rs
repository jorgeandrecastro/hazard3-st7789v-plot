#![no_std]
#![forbid(unsafe_code)]
//! # hazard3-st7789v-plot
//!
//! An adaptive, configurable Cartesian (X, Y) plotting engine for ST7789V TFT LCD
//! displays, built on [`hazard3-st7789v`](https://docs.rs/hazard3-st7789v).
//! This crate is part of [Hazard3 Async](https://github.com/jorgeandrecastro/hazard3-async),
//! a project independent of Embassy. The default backend is `time-hazard3`;
//! to use upstream Embassy, disable default features and enable `time-upstream`.
//! Exactly one backend must be enabled.
//!
//! ## Main features
//!
//! - **`#![no_std]` + `#![forbid(unsafe_code)]`**: Safe and optimized for bare metal.
//! - **No dynamic allocation**: Static buffers only (ring buffer).
//! - **Two-phase rendering**: Frame, labels, and titles are drawn once.
//!   Only the inner grid and signal are refreshed dynamically.
//! - **Configurable axes**: Static ticks with custom labels and automatic zero
//!   detection and coloring.
//! - **Circular history**: Up to 240 points (the screen's physical limit).
//! - **Strict border protection**: Primitives are clamped to the usable interior.
//! - **`hazard3-st7789v` re-exports**: This crate re-exports `Color`, `NoPin`, `St7789v`, `St7789vBuffered`, `SCREEN_H`, `SCREEN_W`, `APPROX`, `DEGREE`, `DELTA`, `DIVIDE`, `GE`, `INFINITY`, `LAMBDA`, `LE`, `PI`, `PLUS_MINUS`, `SQRT`, `THETA`, and `TIMES` from `hazard3_st7789v_plot`.
//! - **Two rendering modes**:
//!
//! | Mode | Type | Flicker |
//! |------|------|:------------:|
//! | [`LineChart`] + [`render()`](LineChart::render) | async, direct SPI | possible |
//! | [`LineChartBuffered`] + [`render_buf()`](LineChartBuffered::render_buf) + `flush()` | sync, RAM | reduced |
//!
//! ## Main types
//!
//! - [`Graphics`]: Async graphics context (direct SPI).
//! - [`AxisConfig`]: Axis configuration (min, max, step, label).
//! - [`PlotConfig`]: Complete configuration (position, margins, colors, axes).
//! - [`LineChart`]: Async chart with direct SPI rendering.
//! - [`LineChartBuffered`]: Synchronous chart with framebuffer rendering.
//!
//! ## Example — async direct mode
//!
//! ```no_run
//! # use hazard3_st7789v_plot::{Color, St7789v, NoPin};
//! # use embedded_hal::digital::OutputPin;
//! # use embedded_hal_async::spi::SpiDevice;
//! use hazard3_st7789v_plot::{Graphics, AxisConfig, PlotConfig, LineChart};
//!
//! # async fn example<SPI, DC>(display: &mut St7789v<SPI, DC, NoPin>)
//! # where SPI: SpiDevice, DC: OutputPin,
//! # {
//! let x_axis = AxisConfig::new(0.0, 10.0, 2.0, b"Time (s)");
//! let y_axis = AxisConfig::new(-50.0, 50.0, 20.0, b"Volt (mV)");
//!
//! let config = PlotConfig {
//!     x: 10, y: 10, width: 220, height: 200,
//!     margin_left: 30, margin_right: 10,
//!     margin_top: 10, margin_bottom: 30,
//!     x_axis, y_axis,
//!     bg_color:    Color::BLACK,
//!     line_color:  Color::GREEN,
//!     axis_color:  Color::WHITE,
//!     grid_color:  Color::GRAY,
//!     text_color:  Color::WHITE,
//!     label_color: Color::CYAN,
//! };
//!
//! let mut chart: LineChart<100> = LineChart::new(config);
//! chart.push(12.4);
//! chart.push(-5.2);
//! chart.push(22.1);
//!
//! let mut gfx = Graphics::new_no_rst(display);
//! chart.render(&mut gfx).await;
//! # }
//! ```
//!
//! ## Example: synchronous framebuffer mode
//!
//! ```no_run
//! # use hazard3_st7789v_plot::{Color, St7789v, St7789vBuffered, NoPin};
//! # use embedded_hal::digital::OutputPin;
//! # use embedded_hal_async::spi::SpiDevice;
//! use hazard3_st7789v_plot::{AxisConfig, PlotConfig, LineChartBuffered};
//!
//! # async fn example<SPI, DC>(spi: SPI, dc: DC, rst: NoPin)
//! # where SPI: SpiDevice, DC: OutputPin,
//! # {
//! let config = PlotConfig {
//!     x: 10, y: 10, width: 220, height: 200,
//!     margin_left: 30, margin_right: 10,
//!     margin_top: 10, margin_bottom: 30,
//!     x_axis: AxisConfig::new(0.0, 10.0, 2.0, b"Time (s)"),
//!     y_axis: AxisConfig::new(-50.0, 50.0, 20.0, b"Volt (mV)"),
//!     bg_color:    Color::BLACK,
//!     line_color:  Color::GREEN,
//!     axis_color:  Color::WHITE,
//!     grid_color:  Color::GRAY,
//!     text_color:  Color::WHITE,
//!     label_color: Color::CYAN,
//! };
//!
//! let mut display = St7789vBuffered::new(St7789v::new(spi, dc, rst));
//! display.driver().init().await.unwrap();
//!
//! let mut chart: LineChartBuffered<100> = LineChartBuffered::new(config);
//!
//! loop {
//!     chart.push(12.4);
//!     chart.render_buf(&mut display);  // synchronous RAM rendering
//!     display.flush().await.unwrap();  // send the framebuffer over SPI
//! }
//! # }
//! ```
//!
//! ## Borrowing pattern: direct mode
//!
//! `Graphics` holds a `&mut St7789v` for its entire lifetime.
//! To call driver methods (text, orientation, etc.),
//! `gfx` must first go out of scope.
//!
//! ```rust,no_run
//! # use hazard3_st7789v_plot::{Color, Graphics, LineChart, NoPin, St7789v};
//! # use embedded_hal::digital::OutputPin;
//! # use embedded_hal_async::spi::SpiDevice;
//! # async fn example<SPI: SpiDevice, DC: OutputPin>(display: &mut St7789v<SPI, DC, NoPin>, chart: &mut LineChart<100>) {
//! {
//!     let mut gfx = Graphics::new_no_rst(display);
//!     chart.render(&mut gfx).await;
//! } // ← borrow released
//! display.draw_str(8, 8, b"OK", Color::WHITE, Color::BLACK).await.unwrap();
//! # }
//! ```
//!
//! ## Borrowing pattern: framebuffer mode
//!
//! `render_buf` takes `&mut St7789vBuffered` directly.
//! Call `flush()` after rendering.
//!
//! ```rust,no_run
//! # use hazard3_st7789v_plot::{LineChartBuffered, NoPin, St7789vBuffered};
//! # use embedded_hal::digital::OutputPin;
//! # use embedded_hal_async::spi::SpiDevice;
//! # async fn example<SPI: SpiDevice, DC: OutputPin>(display: &mut St7789vBuffered<SPI, DC, NoPin>, chart: &mut LineChartBuffered<100>) {
//! chart.render_buf(display);  // sync
//! display.flush().await.unwrap();  // async, single SPI transfer
//! # }
//! ```
//!
//! ## Note on SPI errors
//!
//! This crate's functions silently ignore SPI errors.
//! If your application requires detailed error handling, call
//! `display.draw_pixel()` directly.
//! The `render_buf` primitives never access SPI, so no SPI errors can occur.
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;
/// Re-export the main types for convenient imports.
pub use hazard3_st7789v::{
    Color, NoPin, St7789v, St7789vBuffered, SCREEN_H, SCREEN_W,
    APPROX, DEGREE, DELTA, DIVIDE, GE, INFINITY, LAMBDA, LE, PI, PLUS_MINUS, SQRT, THETA, TIMES,
};
use hazard3_st7789v_graphics::{GraphicsBuffered, line_buf};


/// Maximum chart data history size.
pub const PLOT_HISTORY_LIMIT: usize = 240;

// ─────────────────────────────────────────────────────────────────────────────
// Embedded graphics context
// ─────────────────────────────────────────────────────────────────────────────

pub struct Graphics<'a, SPI, DC, RST = NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Reference to the ST7789V display.
    pub display: &'a mut St7789v<SPI, DC, RST>,
}

impl<'a, SPI, DC> Graphics<'a, SPI, DC, NoPin>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    /// Creates a new graphics context without an RST pin.
    ///
    /// # Arguments
    ///
    /// * `display` - Mutable reference to the ST7789V display
    #[inline]
    pub fn new_no_rst(display: &'a mut St7789v<SPI, DC, NoPin>) -> Self {
        Self { display }
    }
}

impl<'a, SPI, DC, RST> Graphics<'a, SPI, DC, RST>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Creates a new graphics context with an RST pin.
    ///
    /// # Arguments
    ///
    /// * `display` - Mutable reference to the ST7789V display
    #[inline]
    pub fn new(display: &'a mut St7789v<SPI, DC, RST>) -> Self {
        Self { display }
    }

    /// Draws a pixel at (x, y) with the given color.
    ///
    /// Negative or off-screen coordinates are silently ignored.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (may be negative or off-screen)
    /// * `y` - Y coordinate (may be negative or off-screen)
    /// * `color` - Pixel color
 
    #[inline(always)]
    pub async fn pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && x < SCREEN_W as i32 && y < SCREEN_H as i32 {
            let _ = self.display.draw_pixel(x as u16, y as u16, color).await;
        }
    }
}

/// Draws a line between two points using Bresenham's algorithm.
///
/// This function draws a line between (x0, y0) and (x1, y1) using
/// Bresenham's algorithm. Off-screen pixels are handled by the bounds check
/// in [`Graphics::pixel`].
///
/// # Arguments
///
/// * `gfx` - Graphics context
/// * `x0` - Starting point X coordinate
/// * `y0` - Starting point Y coordinate
/// * `x1` - Ending point X coordinate
/// * `y1` - Ending point Y coordinate
/// * `color` - Line color
pub async fn line<SPI, DC, RST>(
    gfx: &mut Graphics<'_, SPI, DC, RST>,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: Color,
) where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        gfx.pixel(x0, y0, color).await;
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Axis configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Defines an axis with fixed static ticks.
///
/// Configures a chart axis with a value range, regular tick spacing,
/// and a descriptive label.
///
/// # Fields
///
/// * `start` - Minimum axis value (e.g. 0.0)
/// * `end` - Maximum axis value (e.g. 10.0)
/// * `step` - Regular spacing between ticks (e.g. 1.0)
/// * `label` - Text label displayed along the axis (e.g. b"Temp (C)")
#[derive(Clone, Copy, Debug)]
pub struct AxisConfig {
    pub start: f32,
    pub end: f32,
    pub step: f32,
    pub label: &'static [u8],
}

impl AxisConfig {
    /// Creates a new axis configuration.
    ///
    /// # Arguments
    ///
    /// * `start` - Minimum value (must be < `end`)
    /// * `end` - Maximum value (must be > `start`)
    /// * `step` - Tick spacing (must be > 0)
    /// * `label` - Static label to display (e.g. b"Temp (C)")
    ///
    /// # Panics
    ///
    /// This does not panic, but use [`is_valid`](Self::is_valid) after construction
    /// to check validity.
    pub const fn new(start: f32, end: f32, step: f32, label: &'static [u8]) -> Self {
        Self { start, end, step, label }
    }

    /// Checks the validity of the configuration.
    ///
    /// Returns `true` if:
    /// - `step` > 0.0
    /// - `end` > `start`
    ///

    pub fn is_valid(&self) -> bool {
        self.step > 0.0 && self.end > self.start
    }

    /// Calculates the number of ticks (including start and end).
    ///
    /// In `no_std`, a direct cast replaces `f32::floor()`.
    ///
    /// # Returns
    ///
    /// The number of ticks including the endpoints, or 0 if the configuration is invalid.

    pub fn tick_count(&self) -> usize {
        if !self.is_valid() {
            return 0;
        }
        let count = ((self.end - self.start) / self.step) as usize;
        count + 1
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plot configuration and structure
// ─────────────────────────────────────────────────────────────────────────────

/// Complete chart configuration.
///
/// Defines all visual and geometric chart parameters:
/// position, size, margins, axes, and color palette.
///
/// # Fields
///
/// * `x`, `y` - Position of the chart's top-left corner (in pixels)
/// * `width`, `height` - Chart dimensions (in pixels)
/// * `margin_*` - Inner margins for axes and labels
/// * `x_axis`, `y_axis` - Configurations for both axes
/// * `*_color` - Colors for the background, lines, axes, grid, text, and labels

#[derive(Clone, Copy, Debug)]
pub struct PlotConfig {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub margin_left: i32,
    pub margin_right: i32,
    pub margin_top: i32,
    pub margin_bottom: i32,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    pub bg_color: Color,
    pub line_color: Color,
    pub axis_color: Color,
    pub grid_color: Color,
    pub text_color: Color,
    pub label_color: Color,
}

/// Chart manager with configurable static axes.
///
/// `LineChart<N>` maintains a circular history of N data points and
/// renders the chart with grid, axes, ticks, and labels.
///
/// # Generic parameter
///
/// * `N` - Maximum number of historical points (must be ≤ `PLOT_HISTORY_LIMIT` = 240)
///
/// # Internal operation
///
/// - **Ring buffer**: Data is stored in a fixed array with a rotating `head`
///   pointer. When the buffer is full, new data overwrites the oldest data.
/// - **History**: Only the N most recent points are displayed.
/// - **Rendering**: Data is converted to pixels via `scale_x()` and `scale_y()`,
///   then connected by Bresenham lines.
pub struct LineChart<const N: usize> {
    config: PlotConfig,
    data: [f32; N],
    head: usize,
    count: usize,
    plot_x: i32,
    plot_y: i32,
    plot_w: i32,
    plot_h: i32,
    initialized: bool, // Tracks the chart's initialization state
}

impl<const N: usize> LineChart<N> {
    /// Creates a new chart manager.
    ///
    /// # Arguments
    ///
    /// * `config` - Complete chart configuration
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - N > `PLOT_HISTORY_LIMIT` (240)
    /// - The X-axis configuration is invalid
    /// - The Y-axis configuration is invalid

    pub fn new(config: PlotConfig) -> Self {
        assert!(N <= PLOT_HISTORY_LIMIT, "History exceeds the physical horizontal limit.");
        assert!(config.x_axis.is_valid(), "Invalid X-axis configuration.");
        assert!(config.y_axis.is_valid(), "Invalid Y-axis configuration.");
        
        let plot_x = config.x + config.margin_left;
        let plot_y = config.y + config.margin_top;
        let plot_w = config.width - config.margin_left - config.margin_right;
        let plot_h = config.height - config.margin_top - config.margin_bottom;

        Self {
            config,
            data: [0.0; N],
            head: 0,
            count: 0,
            plot_x,
            plot_y,
            plot_w,
            plot_h,
            initialized: false, // Initialized to false by default
        }
    }

    /// Returns a reference to the chart configuration.
    #[inline]
    pub fn config(&self) -> &PlotConfig {
        &self.config
    }

    /// Adds a new value to the history.
    ///
    /// If the buffer is full (N points), the oldest value is replaced.
    ///
    /// # Arguments
    ///
    /// * `value` - Value to add (clamped to the Y-axis range during rendering)

    pub fn push(&mut self, value: f32) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % N;
        if self.count < N {
            self.count += 1;
        }
    }

    /// Clears the history and resets the chart.
    pub fn clear(&mut self) {
        self.head = 0;
        self.count = 0;
        self.data.fill(0.0);
        self.initialized = false; // Redraw the frame on the next render
    }

    #[inline]
    fn get_sample(&self, index: usize) -> f32 {
        let oldest = if self.count < N { 0 } else { self.head };
        self.data[(oldest + index) % N]
    }

    /// Converts a Y value to a screen coordinate (pixels).
    ///
    /// The value is clamped to the [y_min, y_max] range defined by `y_axis`,
    /// then converted linearly to pixels.
    #[inline]
    fn scale_y(&self, value: f32) -> i32 {
        let y_min = self.config.y_axis.start;
        let y_max = self.config.y_axis.end;
        
        if y_max <= y_min {
            return self.plot_y + self.plot_h - 1;
        }

        let clamped_val = value.max(y_min).min(y_max);
        let ratio = (clamped_val - y_min) / (y_max - y_min);
        
        self.plot_y + self.plot_h - 1 - (ratio * (self.plot_h - 1) as f32) as i32
    }

    /// Converts a data index to an X screen coordinate (pixels).
    ///
    /// The N points are distributed evenly across the plot area's width.
    #[inline]
    fn scale_x(&self, index: usize) -> i32 {
        if N <= 1 {
            return self.plot_x;
        }
        self.plot_x + (index as i32 * (self.plot_w - 1)) / (N as i32 - 1)
    }

    /// Converts an X-axis value to a screen coordinate (for labels).
    ///
    /// Similar to `scale_y`, but for the X axis.
    #[inline]
    fn scale_x_value(&self, value: f32) -> i32 {
        let x_min = self.config.x_axis.start;
        let x_max = self.config.x_axis.end;
        
        if x_max <= x_min {
            return self.plot_x;
        }
        
        let clamped = value.max(x_min).min(x_max);
        let ratio = (clamped - x_min) / (x_max - x_min);
        self.plot_x + (ratio * (self.plot_w - 1) as f32) as i32
    }

    /// Displays the complete chart with grid, axes, ticks, and curve.
    ///
    /// This method performs:
    /// 1. Background fill
    /// 2. Horizontal (Y) grid and tick labels
    /// 3. Vertical (X) grid and tick labels
    /// 4. Axis labels (titles)
    /// 5. Outer borders
    /// 6. Curve drawing (data)
    ///
    /// # Arguments
    ///
    /// * `gfx` - Initialized graphics context
    ///

   pub async fn render<SPI, DC, RST>(&mut self, gfx: &mut Graphics<'_, SPI, DC, RST>)
    where
        SPI: SpiDevice,
        DC: OutputPin,
        RST: OutputPin,
    {
        let right_edge = self.plot_x + self.plot_w - 1;
        let bottom_edge = self.plot_y + self.plot_h - 1;

        //  1. DRAW FRAME AND TEXT ONCE (first pass only) ──
        if !self.initialized {
            // Clear the entire area allocated to the component
            let _ = gfx.display.fill_rect(
                self.config.x as u16,
                (self.config.y - 16).max(0) as u16,
                (self.config.x + self.config.width) as u16,
                (self.config.y + self.config.height + 16).min(SCREEN_H as i32) as u16,
                self.config.bg_color,
            ).await;

            // Y labels
            let y_axis = &self.config.y_axis;
            let y_range = y_axis.end - y_axis.start;
            let tick_count_y = y_axis.tick_count();

            for i in 0..tick_count_y {
                let value = y_axis.start + (i as f32 * y_axis.step);
                let ratio = (value - y_axis.start) / y_range;
                let y_grid = bottom_edge - (ratio * (self.plot_h - 1) as f32) as i32;

                // Special color for true zero
                let label_color = if value.abs() < 0.001 {
                    Color::GREEN
                } else {
                    self.config.text_color
                };

                let label_y = (y_grid - 4).max(self.config.y + 8).min(self.config.y + self.config.height - 8);

                let _ = gfx.display.draw_f32(
                    (self.config.x + 2) as u16,
                    label_y as u16,
                    value,
                    1,
                    label_color,
                    self.config.bg_color,
                ).await;
            }

            // X labels
            let x_axis = &self.config.x_axis;
            let tick_count_x = x_axis.tick_count();

            for i in 0..tick_count_x {
                let value = x_axis.start + (i as f32 * x_axis.step);
                let x_grid = self.scale_x_value(value);

                let label_x = (x_grid - 8).max(self.config.x + 2).min(self.config.x + self.config.width - 20);

                let _ = gfx.display.draw_f32(
                    label_x as u16,
                    (bottom_edge + 4) as u16,
                    value,
                    1,
                    self.config.text_color,
                    self.config.bg_color,
                ).await;
            }

            // Axis titles
            let _ = gfx.display.draw_str(
                (self.config.x + 2) as u16,
                (self.config.y - 16).max(0) as u16,
                y_axis.label,
                self.config.label_color,
                self.config.bg_color,
            ).await;

            let label_x_x = self.plot_x + (self.plot_w / 2) - ((x_axis.label.len() as i32 * 6) / 2);
            let _ = gfx.display.draw_str(
                label_x_x.max(self.config.x + 2) as u16,
                (self.config.y + self.config.height + 4).min(SCREEN_H as i32 - 8) as u16,
                x_axis.label,
                self.config.label_color,
                self.config.bg_color,
            ).await;

            // Fixed outer borders
            let _ = gfx.display.draw_hline(self.plot_x as u16, self.plot_y as u16, self.plot_w as u16, self.config.axis_color).await;
            let _ = gfx.display.draw_hline(self.plot_x as u16, bottom_edge as u16, self.plot_w as u16, self.config.axis_color).await;
            let _ = gfx.display.draw_vline(self.plot_x as u16, self.plot_y as u16, self.plot_h as u16, self.config.axis_color).await;
            let _ = gfx.display.draw_vline(right_edge as u16, self.plot_y as u16, self.plot_h as u16, self.config.axis_color).await;

            self.initialized = true;
        }

        //  2. DYNAMIC INTERIOR CLEARING (without touching borders) ──
        let _ = gfx.display.fill_rect(
            (self.plot_x + 1) as u16,
            (self.plot_y + 1) as u16,
            (right_edge - 1) as u16,
            (bottom_edge - 1) as u16,
            self.config.bg_color,
        ).await;

        // Inner horizontal (Y) grid
        let y_axis = &self.config.y_axis;
        let y_range = y_axis.end - y_axis.start;
        let tick_count_y = y_axis.tick_count();
        for i in 1..tick_count_y - 1 {
            let value = y_axis.start + (i as f32 * y_axis.step);
            let ratio = (value - y_axis.start) / y_range;
            let y_grid = bottom_edge - (ratio * (self.plot_h - 1) as f32) as i32;

            // Avoid drawing the grid over the one-pixel borders
            if y_grid > self.plot_y && y_grid < bottom_edge {
                // Give the zero line a distinctive color
                let color = if value.abs() < 0.001 { Color::GREEN } else { self.config.grid_color };
                
                let _ = gfx.display.draw_hline(
                    (self.plot_x + 1) as u16, 
                    y_grid as u16, 
                    (self.plot_w - 2) as u16, 
                    color
                ).await;
            }
        }

        // Inner vertical (X) grid
        let x_axis = &self.config.x_axis;
        let tick_count_x = x_axis.tick_count();
        for i in 1..tick_count_x - 1 {
            let value = x_axis.start + (i as f32 * x_axis.step);
            let x_grid = self.scale_x_value(value);

            if x_grid > self.plot_x && x_grid < right_edge {
                let _ = gfx.display.draw_vline(
                    x_grid as u16,
                    (self.plot_y + 1) as u16,
                    (self.plot_h - 2) as u16,
                    self.config.grid_color
                ).await;
            }
        }

        //  3. DRAW THE DATA CURVE (strictly inside the borders) ──
        if self.count < 2 {
            if self.count == 1 {
                let px = self.scale_x(0).max(self.plot_x + 1).min(right_edge - 1);
                let py = self.scale_y(self.get_sample(0)).max(self.plot_y + 1).min(bottom_edge - 1);
                gfx.pixel(px, py, self.config.line_color).await;
            }
            return;
        }

        // Retrieve and clamp points so they do not bleed onto the white border
        let mut prev_x = self.scale_x(0).max(self.plot_x + 1).min(right_edge - 1);
        let mut prev_y = self.scale_y(self.get_sample(0)).max(self.plot_y + 1).min(bottom_edge - 1);

        for i in 1..self.count {
            let next_x = self.scale_x(i).max(self.plot_x + 1).min(right_edge - 1);
            let next_y = self.scale_y(self.get_sample(i)).max(self.plot_y + 1).min(bottom_edge - 1);

            line(gfx, prev_x, prev_y, next_x, next_y, self.config.line_color).await;

            prev_x = next_x;
            prev_y = next_y;
        }
    }

}

 // ─────────────────────────────────────────────────────────────────────────────
// LineChart framebuffer
// ─────────────────────────────────────────────────────────────────────────────

/// Framebuffer version of [`LineChart`].
///
/// Identical to `LineChart`, but all drawing operations write to the RAM
/// framebuffer via `GraphicsBuffered` and the `*_buf` primitives.
/// No pixels are sent to the display until `flush()` is called.
///
/// # Usage pattern
///
/// ```rust,no_run
/// # use hazard3_st7789v_plot::{LineChartBuffered, NoPin, PlotConfig, St7789vBuffered};
/// # use embedded_hal::digital::OutputPin;
/// # use embedded_hal_async::spi::SpiDevice;
/// # async fn example<SPI: SpiDevice, DC: OutputPin>(config: PlotConfig, display: &mut St7789vBuffered<SPI, DC, NoPin>) {
/// let mut chart: LineChartBuffered<100> = LineChartBuffered::new(config);
/// chart.push(12.4);
/// chart.render_buf(display);  // synchronous RAM rendering
/// display.flush().await.unwrap();  // send the framebuffer over SPI
/// # }
/// ```
pub struct LineChartBuffered<const N: usize> {
    config: PlotConfig,
    data: [f32; N],
    head: usize,
    count: usize,
    plot_x: i32,
    plot_y: i32,
    plot_w: i32,
    plot_h: i32,
    initialized: bool,
}

impl<const N: usize> LineChartBuffered<N> {
    pub fn new(config: PlotConfig) -> Self {
        assert!(N <= PLOT_HISTORY_LIMIT);
        assert!(config.x_axis.is_valid());
        assert!(config.y_axis.is_valid());

        let plot_x = config.x + config.margin_left;
        let plot_y = config.y + config.margin_top;
        let plot_w = config.width  - config.margin_left - config.margin_right;
        let plot_h = config.height - config.margin_top  - config.margin_bottom;

        Self { config, data: [0.0; N], head: 0, count: 0,
               plot_x, plot_y, plot_w, plot_h, initialized: false }
    }

    pub fn push(&mut self, value: f32) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % N;
        if self.count < N { self.count += 1; }
    }

    pub fn clear(&mut self) {
        self.head = 0; self.count = 0;
        self.data.fill(0.0);
        self.initialized = false;
    }

    #[inline]
    fn get_sample(&self, index: usize) -> f32 {
        let oldest = if self.count < N { 0 } else { self.head };
        self.data[(oldest + index) % N]
    }

    #[inline]
    fn scale_y(&self, value: f32) -> i32 {
        let y_min = self.config.y_axis.start;
        let y_max = self.config.y_axis.end;
        if y_max <= y_min { return self.plot_y + self.plot_h - 1; }
        let ratio = (value.max(y_min).min(y_max) - y_min) / (y_max - y_min);
        self.plot_y + self.plot_h - 1 - (ratio * (self.plot_h - 1) as f32) as i32
    }

    #[inline]
    fn scale_x(&self, index: usize) -> i32 {
        if N <= 1 { return self.plot_x; }
        self.plot_x + (index as i32 * (self.plot_w - 1)) / (N as i32 - 1)
    }

    #[inline]
    fn scale_x_value(&self, value: f32) -> i32 {
        let x_min = self.config.x_axis.start;
        let x_max = self.config.x_axis.end;
        if x_max <= x_min { return self.plot_x; }
        let ratio = (value.max(x_min).min(x_max) - x_min) / (x_max - x_min);
        self.plot_x + (ratio * (self.plot_w - 1) as f32) as i32
    }

    /// Renders the complete chart to the framebuffer — **synchronously, without `.await`**.
    ///
    /// Call `display.flush().await` afterwards to send it to the display.
    pub fn render_buf<SPI, DC, RST>(
        &mut self,
        display: &mut St7789vBuffered<SPI, DC, RST>,
    ) where
        SPI: embedded_hal_async::spi::SpiDevice,
        DC: embedded_hal::digital::OutputPin,
        RST: embedded_hal::digital::OutputPin,
    {
        let right_edge  = self.plot_x + self.plot_w - 1;
        let bottom_edge = self.plot_y + self.plot_h - 1;

        // ── 1. Frame, labels, titles — once ───────────────────────────────
        if !self.initialized {
            // Background
            display.fill_rect_buf(
                self.config.x as u16,
                (self.config.y - 16).max(0) as u16,
                (self.config.x + self.config.width) as u16,
                (self.config.y + self.config.height + 16).min(SCREEN_H as i32) as u16,
                self.config.bg_color,
            );

            // Y labels
            let y_axis = &self.config.y_axis;
            let y_range = y_axis.end - y_axis.start;
            for i in 0..y_axis.tick_count() {
                let value = y_axis.start + i as f32 * y_axis.step;
                let ratio = (value - y_axis.start) / y_range;
                let y_grid = bottom_edge - (ratio * (self.plot_h - 1) as f32) as i32;
                let label_y = (y_grid - 4).max(self.config.y + 8).min(self.config.y + self.config.height - 8);
                let color = if value.abs() < 0.001 { Color::GREEN } else { self.config.text_color };
                // Render the f32 label using draw_f32_buf (see below)
                self.draw_f32_buf(display, (self.config.x + 2) as u16, label_y as u16, value, 1, color);
            }

            // X labels
            let x_axis = &self.config.x_axis;
            for i in 0..x_axis.tick_count() {
                let value = x_axis.start + i as f32 * x_axis.step;
                let x_grid = self.scale_x_value(value);
                let label_x = (x_grid - 8).max(self.config.x + 2).min(self.config.x + self.config.width - 20);
                self.draw_f32_buf(display, label_x as u16, (bottom_edge + 4) as u16, value, 1, self.config.text_color);
            }

            // Titles
            display.draw_str_buf(
                (self.config.x + 2) as u16,
                (self.config.y - 16).max(0) as u16,
                y_axis.label,
                self.config.label_color,
                self.config.bg_color,
            );
            let label_x_x = self.plot_x + (self.plot_w / 2) - (x_axis.label.len() as i32 * 6 / 2);
            display.draw_str_buf(
                label_x_x.max(self.config.x + 2) as u16,
                (self.config.y + self.config.height + 4).min(SCREEN_H as i32 - 8) as u16,
                x_axis.label,
                self.config.label_color,
                self.config.bg_color,
            );

            // Borders
            for px in self.plot_x..=right_edge {
                display.set_pixel(px as u16, self.plot_y as u16,   self.config.axis_color);
                display.set_pixel(px as u16, bottom_edge as u16,   self.config.axis_color);
            }
            for py in self.plot_y..=bottom_edge {
                display.set_pixel(self.plot_x as u16,  py as u16,  self.config.axis_color);
                display.set_pixel(right_edge as u16,   py as u16,  self.config.axis_color);
            }

            self.initialized = true;
        }

        // ── 2. Clear interior ─────────────────────────────────────────────
        display.fill_rect_buf(
            (self.plot_x + 1) as u16,
            (self.plot_y + 1) as u16,
            (right_edge  - 1) as u16,
            (bottom_edge - 1) as u16,
            self.config.bg_color,
        );

        // ── 3. Y grid ─────────────────────────────────────────────────────
        let y_axis = &self.config.y_axis;
        let y_range = y_axis.end - y_axis.start;
        for i in 1..y_axis.tick_count().saturating_sub(1) {
            let value = y_axis.start + i as f32 * y_axis.step;
            let ratio = (value - y_axis.start) / y_range;
            let y_grid = bottom_edge - (ratio * (self.plot_h - 1) as f32) as i32;
            if y_grid > self.plot_y && y_grid < bottom_edge {
                let color = if value.abs() < 0.001 { Color::GREEN } else { self.config.grid_color };
                display.fill_rect_buf(
                    (self.plot_x + 1) as u16, y_grid as u16,
                    (right_edge  - 1) as u16, y_grid as u16,
                    color,
                );
            }
        }

        // ── 4. X grid ─────────────────────────────────────────────────────
        let x_axis = &self.config.x_axis;
        for i in 1..x_axis.tick_count().saturating_sub(1) {
            let value = x_axis.start + i as f32 * x_axis.step;
            let x_grid = self.scale_x_value(value);
            if x_grid > self.plot_x && x_grid < right_edge {
                display.fill_rect_buf(
                    x_grid as u16, (self.plot_y + 1) as u16,
                    x_grid as u16, (bottom_edge - 1) as u16,
                    self.config.grid_color,
                );
            }
        }

        // ── 5. Curve ──────────────────────────────────────────────────────
        if self.count < 2 {
            if self.count == 1 {
                let px = self.scale_x(0).max(self.plot_x + 1).min(right_edge - 1);
                let py = self.scale_y(self.get_sample(0)).max(self.plot_y + 1).min(bottom_edge - 1);
                display.set_pixel(px as u16, py as u16, self.config.line_color);
            }
            return;
        }

        let mut gfx = GraphicsBuffered::new(display);
        let mut prev_x = self.scale_x(0).max(self.plot_x + 1).min(right_edge - 1);
        let mut prev_y = self.scale_y(self.get_sample(0)).max(self.plot_y + 1).min(bottom_edge - 1);

        for i in 1..self.count {
            let next_x = self.scale_x(i).max(self.plot_x + 1).min(right_edge - 1);
            let next_y = self.scale_y(self.get_sample(i)).max(self.plot_y + 1).min(bottom_edge - 1);
            line_buf(&mut gfx, prev_x, prev_y, next_x, next_y, self.config.line_color);
            prev_x = next_x;
            prev_y = next_y;
        }
    }

    /// Renders an f32 to the framebuffer (`draw_f32` equivalent for St7789vBuffered).
    fn draw_f32_buf<SPI, DC, RST>(
        &self,
        display: &mut St7789vBuffered<SPI, DC, RST>,
        x: u16, y: u16,
        val: f32,
        decimals: u8,
        fg: Color,
    ) where
        SPI: embedded_hal_async::spi::SpiDevice,
        DC: embedded_hal::digital::OutputPin,
        RST: embedded_hal::digital::OutputPin,
    {
        let bg = self.config.bg_color;
        if val.is_nan()      { display.draw_str_buf(x, y, b"NaN",  fg, bg); return; }
        if val.is_infinite() {
            if val > 0.0 { display.draw_str_buf(x, y, b"+Inf", fg, bg); }
            else         { display.draw_str_buf(x, y, b"-Inf", fg, bg); }
            return;
        }
        let negative = val < 0.0;
        let mut abs = if negative { -val } else { val };
        let mut cx = x;
        if negative { cx = display.draw_char_buf(cx, y, 10, fg, bg); } // '-'
        let factor = { let mut f = 1u32; for _ in 0..decimals { f *= 10; } f };
        abs += 0.5 / factor as f32;
        let integer = abs as u32;
        cx = display.draw_u32_buf(cx, y, integer, fg, bg);
        if decimals > 0 {
            cx = display.draw_char_buf(cx, y, 38, fg, bg); // '.'
            let mut frac = abs - integer as f32;
            let mut digits = [0u8; 8];
            for i in 0..decimals as usize { frac *= 10.0; digits[i] = frac as u8; frac -= digits[i] as f32; }
            for i in 0..decimals as usize { cx = display.draw_char_buf(cx, y, digits[i] as usize, fg, bg); }
        }
        let _ = cx;
    }
}
