# embassy-st7789v-plot

Moteur de tracé de graphiques cartésiens (X, Y) adaptatifs et configurables pour écrans TFT LCD **ST7789V 240×320**, construit au-dessus de [`embassy-st7789v`](https://crates.io/crates/embassy-st7789v).

## Caractéristiques

- **`#![no_std]` + `#![forbid(unsafe_code)]`** : Entièrement sûr et embarqué
- **Zéro allocation dynamique** : Buffers statiques uniquement (ring buffer fixe)
- **Historique circulaire** : Jusqu'à 240 points (limite physique écran 240px)
- **Axes configurables** : Graduations statiques avec pas personnalisable
- **Grille configurable** : Grille horizontale/verticale avec labels personnalisés
- **Détection automatique du zéro** : La ligne zéro est surlignée en vert
- **Protection stricte des bordures** : Clamping des primitives à l'espace interne
- **Rendu ligne Bresenham** : Courbes lisses entre points de données
- **Rendu intelligent (Double Phase)** : Cadre, étiquettes et titres tracés une seule fois
- **Deux modes de rendu** :
  - `LineChart` + `render()` : async, direct SPI
  - `LineChartBuffered` + `render_buf()` + `flush()` : sync RAM, **zéro clignotement**

---

## Installation

```toml
[dependencies]
embassy-st7789v          = "0.5"
embassy-st7789v-graphics = "0.4"
embassy-st7789v-plot     = "0.4"
embedded-hal             = "1.0"
embedded-hal-async       = "1.0"
```

Ce crate ré-exporte également des éléments de `embassy-st7789v`, notamment `Color`, `NoPin`, `St7789v`, `St7789vBuffered`, `SCREEN_H`, `SCREEN_W`, `APPROX`, `DEGREE`, `DELTA`, `DIVIDE`, `GE`, `INFINITY`, `LAMBDA`, `LE`, `PI`, `PLUS_MINUS`, `SQRT`, `THETA` et `TIMES`.

### Exemple : texte avec symboles

```rust
use embassy_st7789v::{Color, St7789v, St7789vBuffered, SCREEN_H};
use embassy_st7789v_plot::{DEGREE, INFINITY, PI};

let mut ecran = St7789vBuffered::new(St7789v::new(spi_dev, dc, rst));
let bg = Color::BLACK;
let H = SCREEN_H as u16;

// ---- TEXTE AVEC SYMBOLES ----
let _ = ecran.draw_str_scaled_buf(20, 2, b"GRAPHICS DEMO", Color::CYAN, bg, 2);
let _ = ecran.draw_str_scaled_buf(2, H as u16 - 48, b"Ligne  Cercle  Triangle  Ellipse  Bezier", Color::WHITE, bg, 1);

// Afficher ∞
let _ = ecran.draw_str_scaled_buf(2, H as u16 - 32, b"INFINI ", Color::WHITE, bg, 1);
let _ = ecran.draw_str_scaled_buf(2 + 7 * 6, H as u16 - 32, &[INFINITY], Color::YELLOW, bg, 1);

// Afficher π
let _ = ecran.draw_str_scaled_buf(2, H as u16 - 20, b"PI ", Color::WHITE, bg, 1);
let _ = ecran.draw_str_scaled_buf(2 + 3 * 6, H as u16 - 20, &[PI], Color::YELLOW, bg, 1);

// Afficher °
let _ = ecran.draw_str_scaled_buf(2 + 4 * 6 + 6, H as u16 - 20, b"DEG ", Color::WHITE, bg, 1);
let _ = ecran.draw_str_scaled_buf(2 + 4 * 6 + 6 + 4 * 6, H as u16 - 20, &[DEGREE], Color::YELLOW, bg, 1);
```

---

## Utilisation rapide

### Mode direct async (`LineChart`)

Simple mais peut produire un clignotement si le graphique est redessiné fréquemment.

```rust
use embassy_st7789v::{Color, St7789v, NoPin};
use embassy_st7789v_plot::{Graphics, AxisConfig, PlotConfig, LineChart};

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

### Mode framebuffer sync (`LineChartBuffered`) zéro clignotement

Toutes les opérations de dessin écrivent en RAM. Un seul `flush()` envoie tout à l'écran.
Nécessite `St7789vBuffered` du crate `embassy-st7789v` (150 Ko RAM).

```rust
use embassy_st7789v::{Color, St7789v, St7789vBuffered};
use embassy_st7789v_plot::{AxisConfig, PlotConfig, LineChartBuffered};

let mut ecran = St7789vBuffered::new(St7789v::new(spi_dev, dc, rst));
ecran.driver().init().await.unwrap();

let config = PlotConfig { /* ... */ };
let mut chart: LineChartBuffered<100> = LineChartBuffered::new(config);

loop {
    let value = sensor.read().await;
    chart.push(value);

    chart.render_buf(&mut ecran);   // tout en RAM, synchrone
    ecran.flush().await.unwrap();   // envoi unique → zéro clignotement
}
```

---

## Configuration des axes

```rust
// Temps 0–60 s, graduation tous les 10 s
let time_axis = AxisConfig::new(0.0, 60.0, 10.0, b"Time (s)");

// Température -10 à +50 °C, graduation tous les 10 °C
let temp_axis = AxisConfig::new(-10.0, 50.0, 10.0, b"Temp (C)");

// Tension 0–3.3 V, graduation tous les 0.5 V
let volt_axis = AxisConfig::new(0.0, 3.3, 0.5, b"U (V)");

// Nombre de graduations
assert_eq!(time_axis.tick_count(), 7); // 0, 10, 20, 30, 40, 50, 60
```

---

## Schéma de positionnement
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

## Workflow de rendu (Double Phase)

**Phase d'initialisation (une seule fois) :**
1. Nettoyage global de l'espace d'affichage
2. Labels Y — valeurs des graduations (vert si zéro détecté)
3. Labels X — valeurs des graduations
4. Titres des axes
5. Bordures fixes (axis_color)

**Phase dynamique (à chaque rendu) :**
1. Nettoyage intérieur strict (sans toucher aux bordures)
2. Grille Y — lignes horizontales (vert pour le zéro)
3. Grille X — lignes verticales
4. Courbe — lignes Bresenham reliant les points

---

## Ring buffer (historique circulaire)
array[100], head=3, count=100 (plein)
data[0] = 97e valeur
data[1] = 98e valeur
data[2] = 99e valeur  ← head (prochaine écriture)
data[3] = 1ère valeur ← oldest
...

---

## Cas d'usage

### Oscilloscope 1 canal

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

### Moniteur de température

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

### Graphique de pression

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

## API complète

### `LineChart<N>` et `LineChartBuffered<N>`

| Méthode | `LineChart` | `LineChartBuffered` |
|---|---|---|
| `new(config)` | ✓ | ✓ |
| `push(value)` | ✓ | ✓ |
| `clear()` | ✓ | ✓ |
| `config()` | ✓ | — |
| `render(&mut gfx)` | ✓ async | — |
| `render_buf(&mut ecran)` | — | ✓ sync |

### `AxisConfig`

| Méthode | Description |
|---|---|
| `new(start, end, step, label)` | Crée une config d'axe |
| `is_valid()` | Vérifie cohérence |
| `tick_count()` | Nombre de graduations |

### `Graphics<'a, SPI, DC, RST>`

| Méthode | Description |
|---|---|
| `new(display)` | Crée contexte avec RST |
| `new_no_rst(display)` | Crée contexte sans RST |
| `pixel(x, y, color)` | Trace pixel (async) |

### Fonctions globales

| Fonction | Mode | Description |
|---|---|---|
| `line(gfx, x0, y0, x1, y1, color)` | async | Bresenham direct SPI |

---

## Limitations

- **Nombre de points** : Maximum 240 (largeur physique écran)
- **Précision** : Axes en virgule flottante, pixels en entier
- **Pas non-adaptatif** : Graduations statiques, pas de zoom automatique
- **RAM framebuffer** : `LineChartBuffered` nécessite 150 Ko (RP2350 ✓, RP2040 ✓, STM32F103 ✗)

---

## Licence

GPL-2.0-or-later — **Copyright (C) 2026 Jorge Andre Castro**

---

## Dépendances

- [`embassy-st7789v`](https://crates.io/crates/embassy-st7789v) — Driver ST7789V + framebuffer
- [`embassy-st7789v-graphics`](https://crates.io/crates/embassy-st7789v-graphics) — Primitives géométriques
- [`embedded-hal`](https://crates.io/crates/embedded-hal) — Traits HAL
- [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async) — Traits async HAL