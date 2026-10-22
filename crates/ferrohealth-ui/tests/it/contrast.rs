// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The contrast of every text and ground pair of the stylesheet, in the
//! light, the chosen dark and the system dark theme of every brand,
//! computed from the custom properties as WCAG 2.2 defines it: 4.5:1 for
//! text (SC 1.4.3) and 3:1 for the edge of a control and the focus ring
//! (SC 1.4.11).
//!
//! A brand's theme file is laid over the stylesheet's role defaults, block
//! by block, as the browser cascades it. A translucent ground is composited
//! over each opaque ground and the worst of them counts, so a tint is
//! measured wherever a component can place it.
//! <https://www.w3.org/TR/WCAG22/#dfn-contrast-ratio>,
//! <https://www.w3.org/TR/WCAG22/#dfn-relative-luminance>

use std::collections::BTreeMap;

use ferrohealth_ui::style::{Brand, STYLESHEET};

/// The text minimum of WCAG 2.2 SC 1.4.3.
const TEXT: f64 = 4.5;

/// The non-text minimum of WCAG 2.2 SC 1.4.11.
const EDGE: f64 = 3.0;

/// The opaque grounds a component paints a region with.
const OPAQUE: &[&str] = &[
    "surface",
    "surface-raised",
    "surface-sunken",
    "surface-head",
    "neutral-ground",
];

/// The translucent or tinted grounds, laid over an opaque one.
const TINTED: &[&str] = &[
    "accent-tint",
    "ok-ground",
    "warn-ground",
    "bad-ground",
    "info-ground",
];

/// Every ground body text can sit on.
const EVERY_GROUND: &[&str] = &[
    "surface",
    "surface-raised",
    "surface-sunken",
    "surface-head",
    "neutral-ground",
    "accent-tint",
    "ok-ground",
    "warn-ground",
    "bad-ground",
    "info-ground",
];

/// The grounds a link, the accent text or the focus ring sits on.
const ACCENT_GROUNDS: &[&str] = &[
    "surface",
    "surface-raised",
    "surface-sunken",
    "surface-head",
    "accent-tint",
];

/// The grounds a highlighted token sits on: the code ground, a line a link
/// targets, and a line a diagnostic marks.
const TOKEN_GROUNDS: &[&str] = &["surface-sunken", "accent-tint", "bad-ground", "warn-ground"];

/// Each text role and the grounds it can sit on.
const TEXT_PAIRS: &[(&str, &[&str])] = &[
    ("ink", EVERY_GROUND),
    ("muted", EVERY_GROUND),
    ("accent", ACCENT_GROUNDS),
    ("accent-hover", ACCENT_GROUNDS),
    ("accent-ink", &["accent", "accent-hover"]),
    ("neutral", OPAQUE),
    ("ok", &["ok-ground", "surface-raised"]),
    ("warn", &["warn-ground", "surface-raised"]),
    ("bad", &["bad-ground", "surface-raised"]),
    ("info", &["info-ground", "surface-raised"]),
    ("syntax-key", TOKEN_GROUNDS),
    ("syntax-string", TOKEN_GROUNDS),
    ("syntax-number", TOKEN_GROUNDS),
    ("syntax-keyword", TOKEN_GROUNDS),
    ("syntax-comment", TOKEN_GROUNDS),
    ("syntax-tag", TOKEN_GROUNDS),
    ("syntax-attribute", TOKEN_GROUNDS),
    ("syntax-path", TOKEN_GROUNDS),
    ("syntax-parameter", TOKEN_GROUNDS),
];

/// Each control edge and the grounds it is drawn against.
const EDGE_PAIRS: &[(&str, &[&str])] = &[
    (
        "edge-control",
        &[
            "surface",
            "surface-raised",
            "surface-sunken",
            "surface-head",
        ],
    ),
    ("focus", ACCENT_GROUNDS),
    ("accent", &["surface", "surface-raised", "surface-sunken"]),
    ("neutral", &["surface-raised"]),
    ("ok", &["surface", "surface-sunken"]),
    ("warn", &["surface"]),
    ("bad", &["surface", "surface-sunken"]),
];

/// An sRGB colour with its alpha, each channel in 0 to 255.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rgba {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

impl Rgba {
    /// This colour laid over an opaque `ground`, blended in sRGB as a
    /// browser composites it.
    fn over(self, ground: Self) -> Self {
        let mix = |top: f64, under: f64| top * self.a + under * (1.0 - self.a);
        Self {
            r: mix(self.r, ground.r),
            g: mix(self.g, ground.g),
            b: mix(self.b, ground.b),
            a: 1.0,
        }
    }

    /// The relative luminance WCAG 2.2 defines.
    fn luminance(self) -> f64 {
        let linear = |c: f64| {
            let s = c / 255.0;
            if s <= 0.040_45 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(self.r) + 0.7152 * linear(self.g) + 0.0722 * linear(self.b)
    }
}

/// The contrast ratio of two opaque colours.
fn ratio(one: Rgba, two: Rgba) -> f64 {
    let (a, b) = (one.luminance(), two.luminance());
    let (light, dark) = if a >= b { (a, b) } else { (b, a) };
    (light + 0.05) / (dark + 0.05)
}

/// `css` with every comment removed.
fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some((before, after)) = rest.split_once("/*") {
        out.push_str(before);
        rest = after.split_once("*/").map_or("", |(_, tail)| tail);
    }
    out.push_str(rest);
    out
}

/// The bodies of every rule whose selector is `selector`, each up to its
/// closing brace (a block of custom properties nests none), joined as one;
/// empty when the stylesheet has none.
fn block(css: &str, selector: &str) -> String {
    let header = format!("{selector} {{");
    css.split(&header)
        .skip(1)
        .map(|body| body.split_once('}').expect("the block should close").0)
        .collect::<Vec<_>>()
        .join(";")
}

/// The colour `value` writes, `#rrggbb` or `rgb(r g b / a)`; `None` for a
/// value that is no colour.
fn parse_colour(value: &str) -> Option<Rgba> {
    if let Some(hex) = value.strip_prefix('#') {
        assert_eq!(hex.len(), 6, "a colour token is written #rrggbb: {value}");
        let channel = |at: usize| {
            let digits = hex.get(at..at + 2).expect("two hex digits");
            f64::from(u8::from_str_radix(digits, 16).expect("a hex channel"))
        };
        return Some(Rgba {
            r: channel(0),
            g: channel(2),
            b: channel(4),
            a: 1.0,
        });
    }
    let inner = value.strip_prefix("rgb(")?.strip_suffix(')')?;
    let (channels, alpha) = inner.split_once('/').unwrap_or((inner, "1"));
    let channels: Vec<f64> = channels
        .split_whitespace()
        .map(|c| c.parse().expect("an rgb() channel"))
        .collect();
    assert_eq!(channels.len(), 3, "rgb() has three channels: {value}");
    Some(Rgba {
        r: channels[0],
        g: channels[1],
        b: channels[2],
        a: alpha.trim().parse().expect("an rgb() alpha"),
    })
}

/// Every colour custom property of one theme block.
fn colours(body: &str) -> BTreeMap<String, Rgba> {
    body.split(';')
        .filter_map(|declaration| {
            let (name, value) = declaration.trim().split_once(':')?;
            let name = name.trim().strip_prefix("--")?;
            Some((name.to_owned(), parse_colour(value.trim())?))
        })
        .collect()
}

/// The three selectors a theme is written under.
const BLOCKS: [(&str, &str); 3] = [
    ("light", ":root"),
    ("dark", ":root[data-theme=\"dark\"]"),
    (
        "system dark",
        ":root:not([data-theme=\"light\"], [data-theme=\"dark\"])",
    ),
];

/// The accents a brand file declares for the theme previews in Your
/// preferences, outside its three theme blocks: the light one and the dark one.
const PREVIEW: [&str; 2] = ["preview-accent-light", "preview-accent-dark"];

/// The three themes of `brand`: the stylesheet's roles with the brand laid
/// over them, block by block, without the preview accents, which no theme
/// paints with.
fn themes(brand: Option<Brand>) -> Vec<(String, BTreeMap<String, Rgba>)> {
    let base = strip_comments(STYLESHEET);
    let over = brand.map(|brand| strip_comments(brand.theme()));
    BLOCKS
        .iter()
        .map(|(name, selector)| {
            let mut theme = colours(&block(&base, selector));
            if let Some(over) = &over {
                theme.extend(colours(&block(over, selector)));
            }
            theme.retain(|name, _| !PREVIEW.contains(&name.as_str()));
            let label = match brand {
                Some(brand) => format!("{} {name}", brand.product()),
                None => format!("default {name}"),
            };
            (label, theme)
        })
        .collect()
}

/// The colour `--name` of `theme`.
fn token(theme: &BTreeMap<String, Rgba>, name: &str) -> Rgba {
    *theme
        .get(name)
        .unwrap_or_else(|| panic!("the theme should define --{name}"))
}

/// The worst ratio of `fg` against `ground`, a tinted ground laid over each
/// opaque one in turn.
fn worst(theme: &BTreeMap<String, Rgba>, fg: &str, ground: &str) -> f64 {
    let fg = token(theme, fg);
    let ground_colour = token(theme, ground);
    let unders: Vec<Rgba> = if TINTED.contains(&ground) {
        OPAQUE
            .iter()
            .map(|under| ground_colour.over(token(theme, under)))
            .collect()
    } else {
        assert!(
            (ground_colour.a - 1.0).abs() < f64::EPSILON,
            "--{ground} is translucent and should be listed as a tinted ground"
        );
        vec![ground_colour]
    };
    unders
        .into_iter()
        .map(|under| ratio(fg.over(under), under))
        .fold(f64::INFINITY, f64::min)
}

/// Every theme the library ships: its defaults and each brand's.
fn every_theme() -> Vec<(String, BTreeMap<String, Rgba>)> {
    std::iter::once(None)
        .chain(Brand::ALL.into_iter().map(Some))
        .flat_map(themes)
        .collect()
}

#[test]
fn the_dark_blocks_agree_and_every_theme_defines_every_role() {
    for brand in std::iter::once(None).chain(Brand::ALL.into_iter().map(Some)) {
        let themes = themes(brand);
        let [(_, light), (_, dark), (_, system)] = themes.as_slice() else {
            panic!("three theme blocks");
        };
        assert_eq!(
            dark, system,
            "{brand:?}: the chosen and the system dark agree"
        );
        let names = |t: &BTreeMap<String, Rgba>| t.keys().cloned().collect::<Vec<_>>();
        assert_eq!(
            names(light),
            names(dark),
            "{brand:?}: each theme defines every role"
        );
    }
}

#[test]
fn a_brand_sets_its_accent_alone() {
    let allowed = [
        "accent",
        "accent-hover",
        "accent-ink",
        "accent-tint",
        "focus",
        PREVIEW[0],
        PREVIEW[1],
    ];
    for brand in Brand::ALL {
        for (_, selector) in BLOCKS {
            let set = colours(&block(&strip_comments(brand.theme()), selector));
            assert!(!set.is_empty(), "{brand:?} sets {selector}");
            for name in set.keys() {
                assert!(allowed.contains(&name.as_str()), "{brand:?} sets --{name}");
            }
        }
    }
}

#[test]
fn a_preview_accent_is_the_accent_of_its_theme() {
    for brand in std::iter::once(None).chain(Brand::ALL.into_iter().map(Some)) {
        let css = strip_comments(brand.map_or(STYLESHEET, Brand::theme));
        let previews = colours(&block(&css, BLOCKS[0].1));
        let [(_, light), (_, dark), _] = themes(brand).try_into().expect("three themes");
        for (name, theme) in PREVIEW.into_iter().zip([light, dark]) {
            let preview = previews.get(name);
            assert!(preview.is_some(), "{brand:?} sets no --{name}");
            assert_eq!(
                preview,
                theme.get("accent"),
                "{brand:?}: --{name} is the accent of its theme, so the accent pairs hold it"
            );
        }
    }
}

#[test]
fn every_text_and_ground_pair_meets_wcag_aa_in_every_brand() {
    let mut failures = Vec::new();
    for (name, theme) in every_theme() {
        for (minimum, pairs) in [(TEXT, TEXT_PAIRS), (EDGE, EDGE_PAIRS)] {
            for (fg, grounds) in pairs {
                for ground in *grounds {
                    let measured = worst(&theme, fg, ground);
                    if measured < minimum {
                        failures.push(format!(
                            "{name}: --{fg} on --{ground} is {measured:.2}:1, under {minimum}:1"
                        ));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The bands of a focused segment's ring and what each one meets, pressed or
/// not: the focus colour on the segment's edge meets the group's ground, a
/// hovered neighbour and the band inside it; on a pressed segment that inner
/// band is the accent ink, which meets the accent fill. The ring itself is
/// drawn by the `.fui-segment:focus-visible` rule of the stylesheet.
const SEGMENT_RING: &[(&str, &[&str])] = &[
    ("focus", &["surface-raised", "surface-sunken"]),
    ("accent-ink", &["accent", "focus"]),
];

#[test]
fn a_focused_segments_ring_meets_3_to_1_pressed_or_not_in_every_brand() {
    let css = strip_comments(STYLESHEET);
    let ring = block(
        &css,
        ".fui-segment:focus-visible,\n  .fui-segment:has(> input:focus-visible)",
    );
    assert!(
        ring.contains("outline: 2px solid var(--focus)")
            && ring.contains("box-shadow: inset 0 0 0 4px var(--segment-ring, transparent)"),
        "a focused segment draws the focus colour and the pressed band: {ring}"
    );
    let pressed = block(
        &css,
        ".fui-segment[aria-pressed=\"true\"],\n  .fui-segment[aria-current=\"true\"],\n  .fui-segment:has(> input:checked)",
    );
    assert!(
        pressed.contains("--segment-ring: var(--accent-ink)")
            && pressed.contains("background: var(--accent)"),
        "a pressed segment's band is the accent ink on the accent: {pressed}"
    );
    let mut failures = Vec::new();
    for (name, theme) in every_theme() {
        for (band, grounds) in SEGMENT_RING {
            for ground in *grounds {
                let measured = worst(&theme, band, ground);
                if measured < EDGE {
                    failures.push(format!(
                        "{name}: the segment ring's --{band} on --{ground} is {measured:.2}:1, under {EDGE}:1"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_ratio_is_the_wcag_contrast_ratio() {
    let black = parse_colour("#000000").expect("black");
    let white = parse_colour("#ffffff").expect("white");
    assert!((ratio(black, white) - 21.0).abs() < 1e-9);
    assert!((ratio(white, white) - 1.0).abs() < 1e-9);
    let half = parse_colour("rgb(0 0 0 / 0.5)").expect("half black");
    let blended = half.over(white);
    assert!((blended.r - 127.5).abs() < 1e-9 && (blended.a - 1.0).abs() < f64::EPSILON);
}
