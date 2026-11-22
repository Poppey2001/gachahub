//! Visual language of the launcher: palette, fonts, egui style and a few
//! reusable painted widgets (gradient buttons, pills, progress bars, cards).

use eframe::egui::{
    self, epaint::Mesh, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily,
    FontId, Margin, Pos2, Rect, Response, Sense, Shadow, Stroke, StrokeKind, Ui, Vec2, Widget,
};
use std::sync::Arc;

// ---------------------------------------------------------------- palette

pub const BG: Color32 = Color32::from_rgb(6, 8, 13);
pub const BG_RAISED: Color32 = Color32::from_rgb(11, 14, 21);
pub const PANEL: Color32 = Color32::from_rgb(14, 18, 27);
pub const PANEL_HOVER: Color32 = Color32::from_rgb(20, 25, 37);
pub const BORDER: Color32 = Color32::from_rgb(30, 37, 51);
pub const BORDER_SOFT: Color32 = Color32::from_rgb(22, 28, 40);
pub const TEXT: Color32 = Color32::from_rgb(242, 245, 252);
pub const TEXT_DIM: Color32 = Color32::from_rgb(165, 174, 190);
pub const MUTED: Color32 = Color32::from_rgb(112, 122, 140);
pub const FAINT: Color32 = Color32::from_rgb(72, 81, 98);
pub const BRAND: Color32 = Color32::from_rgb(126, 205, 255);
pub const BRAND_2: Color32 = Color32::from_rgb(135, 120, 255);
pub const OK: Color32 = Color32::from_rgb(99, 214, 154);
pub const WARN: Color32 = Color32::from_rgb(232, 182, 96);
pub const ERR: Color32 = Color32::from_rgb(255, 110, 120);

pub fn hex(raw: &str) -> Color32 {
    let raw = raw.trim_start_matches('#');
    let p = |i: usize| u8::from_str_radix(raw.get(i..i + 2).unwrap_or("80"), 16).unwrap_or(128);
    if raw.len() >= 6 {
        Color32::from_rgb(p(0), p(2), p(4))
    } else {
        BRAND
    }
}

pub fn alpha(c: Color32, a: f32) -> Color32 {
    let [r, g, b, _] = c.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, (a.clamp(0.0, 1.0) * 255.0) as u8)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let [ar, ag, ab, aa] = a.to_srgba_unmultiplied();
    let [br, bg, bb, ba] = b.to_srgba_unmultiplied();
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(l(ar, br), l(ag, bg), l(ab, bb), l(aa, ba))
}

/// Pick black or white text for a given background.
pub fn on_color(bg: Color32) -> Color32 {
    let [r, g, b, _] = bg.to_srgba_unmultiplied();
    let luma = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    if luma > 150.0 {
        Color32::from_rgb(7, 10, 15)
    } else {
        Color32::WHITE
    }
}

// ---------------------------------------------------------------- fonts

pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}
pub fn heavy() -> FontFamily {
    FontFamily::Name("heavy".into())
}
pub fn font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}
pub fn font_semi(size: f32) -> FontId {
    FontId::new(size, semibold())
}
pub fn font_heavy(size: f32) -> FontId {
    FontId::new(size, heavy())
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "inter".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "inter-semibold".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-SemiBold.ttf"
        ))),
    );
    fonts.font_data.insert(
        "inter-heavy".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-ExtraBold.ttf"
        ))),
    );
    // Keep egui's symbol/emoji fonts as fallback so icons like ▶ ⚙ ⬇ keep working.
    let fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let chain = |first: &str| {
        let mut v = vec![first.to_string()];
        v.extend(fallback.iter().cloned());
        v
    };
    fonts
        .families
        .insert(FontFamily::Proportional, chain("inter"));
    fonts.families.insert(semibold(), chain("inter-semibold"));
    fonts.families.insert(heavy(), chain("inter-heavy"));
    ctx.set_fonts(fonts);

    ctx.set_theme(egui::Theme::Dark);
    ctx.global_style_mut(|style| {
        use egui::TextStyle::*;
        style.text_styles = [
            (Small, font(11.0)),
            (Body, font(13.5)),
            (Monospace, FontId::monospace(12.5)),
            (Button, font_semi(13.0)),
            (Heading, font_heavy(26.0)),
        ]
        .into();
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 7.0);
        style.spacing.interact_size.y = 30.0;
        style.spacing.combo_height = 260.0;
        style.spacing.scroll.bar_width = 6.0;
        style.spacing.scroll.floating = true;
        style.interaction.selectable_labels = false;

        let v = &mut style.visuals;
        v.dark_mode = true;
        v.panel_fill = BG;
        v.window_fill = BG_RAISED;
        v.extreme_bg_color = Color32::from_rgb(9, 12, 18);
        v.faint_bg_color = PANEL;
        v.override_text_color = None;
        v.hyperlink_color = BRAND;
        v.selection.bg_fill = alpha(BRAND, 0.35);
        v.selection.stroke = Stroke::new(1.0, BRAND);
        v.window_corner_radius = CornerRadius::same(18);
        v.menu_corner_radius = CornerRadius::same(12);
        v.window_stroke = Stroke::new(1.0, BORDER);
        v.window_shadow = Shadow {
            offset: [0, 18],
            blur: 48,
            spread: 0,
            color: Color32::from_black_alpha(160),
        };
        v.popup_shadow = Shadow {
            offset: [0, 10],
            blur: 28,
            spread: 0,
            color: Color32::from_black_alpha(140),
        };

        let r = CornerRadius::same(10);
        v.widgets.noninteractive.bg_fill = PANEL;
        v.widgets.noninteractive.weak_bg_fill = PANEL;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SOFT);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
        v.widgets.noninteractive.corner_radius = r;

        v.widgets.inactive.bg_fill = Color32::from_rgb(18, 23, 34);
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(18, 23, 34);
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
        v.widgets.inactive.corner_radius = r;

        v.widgets.hovered.bg_fill = Color32::from_rgb(26, 33, 48);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(26, 33, 48);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(52, 66, 92));
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT);
        v.widgets.hovered.corner_radius = r;
        v.widgets.hovered.expansion = 0.0;

        v.widgets.active.bg_fill = Color32::from_rgb(32, 42, 62);
        v.widgets.active.weak_bg_fill = Color32::from_rgb(32, 42, 62);
        v.widgets.active.bg_stroke = Stroke::new(1.0, alpha(BRAND, 0.6));
        v.widgets.active.fg_stroke = Stroke::new(1.5, TEXT);
        v.widgets.active.corner_radius = r;
        v.widgets.active.expansion = 0.0;

        v.widgets.open = v.widgets.active;
    });
}

// ---------------------------------------------------------------- painting helpers

/// Linear gradient rectangle (left→right when `horizontal`, else top→bottom).
pub fn gradient_rect(
    painter: &egui::Painter,
    rect: Rect,
    a: Color32,
    b: Color32,
    horizontal: bool,
) {
    let mut mesh = Mesh::default();
    let (c0, c1, c2, c3) = if horizontal {
        (a, b, b, a)
    } else {
        (a, a, b, b)
    };
    mesh.colored_vertex(rect.left_top(), c0);
    mesh.colored_vertex(rect.right_top(), c1);
    mesh.colored_vertex(rect.right_bottom(), c2);
    mesh.colored_vertex(rect.left_bottom(), c3);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(mesh);
}

/// Multi-stop horizontal gradient.
pub fn gradient_stops(
    painter: &egui::Painter,
    rect: Rect,
    stops: &[(f32, Color32)],
    horizontal: bool,
) {
    for w in stops.windows(2) {
        let (t0, c0) = w[0];
        let (t1, c1) = w[1];
        let r = if horizontal {
            Rect::from_x_y_ranges(
                rect.left() + rect.width() * t0..=rect.left() + rect.width() * t1,
                rect.y_range(),
            )
        } else {
            Rect::from_x_y_ranges(
                rect.x_range(),
                rect.top() + rect.height() * t0..=rect.top() + rect.height() * t1,
            )
        };
        gradient_rect(painter, r, c0, c1, horizontal);
    }
}

/// Soft radial glow: an elliptical triangle fan fading to transparent.
pub fn radial_glow(painter: &egui::Painter, center: Pos2, radius: Vec2, color: Color32) {
    const SEGMENTS: usize = 64;
    const RINGS: usize = 6;
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);
    for ring in 1..=RINGS {
        let t = ring as f32 / RINGS as f32;
        // smooth falloff
        let a = (1.0 - t).powf(1.8);
        let c = alpha(color, a * color.a() as f32 / 255.0);
        for s in 0..SEGMENTS {
            let ang = s as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            mesh.colored_vertex(
                center + Vec2::new(ang.cos() * radius.x * t, ang.sin() * radius.y * t),
                c,
            );
        }
    }
    for s in 0..SEGMENTS as u32 {
        let n = (s + 1) % SEGMENTS as u32;
        mesh.add_triangle(0, 1 + s, 1 + n);
    }
    for ring in 0..(RINGS - 1) as u32 {
        let base = 1 + ring * SEGMENTS as u32;
        let next = base + SEGMENTS as u32;
        for s in 0..SEGMENTS as u32 {
            let n = (s + 1) % SEGMENTS as u32;
            mesh.add_triangle(base + s, next + s, next + n);
            mesh.add_triangle(base + s, next + n, base + n);
        }
    }
    painter.add(mesh);
}

// ---------------------------------------------------------------- frames

pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(alpha(PANEL, 0.92))
        .stroke(Stroke::new(1.0, BORDER_SOFT))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(16))
}

pub fn glass() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(10, 14, 22, 190))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 18),
        ))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::symmetric(14, 12))
}

// ---------------------------------------------------------------- widgets

/// Big gradient call-to-action button (Play / Install).
pub struct CtaButton<'a> {
    label: &'a str,
    icon: &'a str,
    a: Color32,
    b: Color32,
    enabled: bool,
    min_width: f32,
    height: f32,
}

impl<'a> CtaButton<'a> {
    pub fn new(icon: &'a str, label: &'a str, a: Color32, b: Color32) -> Self {
        Self {
            label,
            icon,
            a,
            b,
            enabled: true,
            min_width: 200.0,
            height: 54.0,
        }
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Widget for CtaButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text = format!("{}   {}", self.icon, self.label);
        let galley = ui
            .painter()
            .layout_no_wrap(text, font_heavy(16.0), Color32::WHITE);
        let size = Vec2::new((galley.size().x + 56.0).max(self.min_width), self.height);
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, resp) = ui.allocate_exact_size(size, sense);
        if ui.is_rect_visible(rect) {
            let hover = ui
                .ctx()
                .animate_bool(resp.id, resp.hovered() && self.enabled);
            let p = ui.painter();
            let lift = hover * 1.5;
            let rect = rect.translate(Vec2::new(0.0, -lift));
            // glow
            for i in 0..4 {
                let grow = 4.0 + i as f32 * 5.0;
                p.rect_filled(
                    rect.expand(grow).translate(Vec2::new(0.0, 6.0)),
                    CornerRadius::same((16.0 + grow) as u8),
                    alpha(self.a, (0.05 + hover * 0.03) * (1.0 - i as f32 / 4.0)),
                );
            }
            let (a, b) = if self.enabled {
                (
                    mix(self.a, Color32::WHITE, 0.08 + hover * 0.1),
                    mix(self.b, self.a, 0.25 + hover * 0.1),
                )
            } else {
                (mix(self.a, BG, 0.55), mix(self.b, BG, 0.55))
            };
            let r = CornerRadius::same(16);
            p.rect_filled(rect, r, a);
            p.rect_filled(
                Rect::from_min_max(Pos2::new(rect.center().x, rect.top()), rect.right_bottom()),
                CornerRadius {
                    nw: 0,
                    sw: 0,
                    ne: 16,
                    se: 16,
                },
                b,
            );
            let clip = p.with_clip_rect(rect);
            gradient_rect(&clip, rect.shrink2(Vec2::new(16.0, 0.0)), a, b, true);
            // top sheen
            gradient_rect(
                &clip,
                Rect::from_min_max(
                    rect.left_top() + Vec2::new(10.0, 1.0),
                    Pos2::new(rect.right() - 10.0, rect.center().y),
                ),
                Color32::from_white_alpha(28),
                Color32::TRANSPARENT,
                false,
            );
            let fg = if self.enabled {
                on_color(mix(a, b, 0.5))
            } else {
                alpha(TEXT, 0.5)
            };
            p.text(
                rect.center(),
                Align2::CENTER_CENTER,
                format!("{}   {}", self.icon, self.label),
                font_heavy(16.0),
                fg,
            );
        }
        if self.enabled {
            resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        } else {
            resp
        }
    }
}

/// Secondary translucent button.
pub fn ghost_button(ui: &mut Ui, icon: &str, label: &str) -> Response {
    ghost_button_sized(ui, icon, label, 54.0, false)
}

pub fn ghost_button_sized(
    ui: &mut Ui,
    icon: &str,
    label: &str,
    height: f32,
    active: bool,
) -> Response {
    let text = if icon.is_empty() {
        label.to_string()
    } else {
        format!("{icon}  {label}")
    };
    let galley = ui
        .painter()
        .layout_no_wrap(text.clone(), font_semi(13.5), TEXT);
    let size = Vec2::new(galley.size().x + 36.0, height);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
        let p = ui.painter();
        let fill = if active {
            alpha(BRAND, 0.16 + hover * 0.06)
        } else {
            Color32::from_rgba_unmultiplied(16, 21, 31, (170.0 + hover * 60.0) as u8)
        };
        let stroke = if active {
            alpha(BRAND, 0.55)
        } else {
            Color32::from_white_alpha((18.0 + hover * 22.0) as u8)
        };
        p.rect(
            rect,
            CornerRadius::same(14),
            fill,
            Stroke::new(1.0, stroke),
            StrokeKind::Inside,
        );
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            font_semi(13.5),
            mix(TEXT_DIM, TEXT, hover.max(active as u8 as f32)),
        );
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Small rounded label chip.
pub fn pill(ui: &mut Ui, text: &str, color: Color32) -> Response {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font_semi(11.0), color);
    let size = galley.size() + Vec2::new(18.0, 10.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter();
    p.rect(
        rect,
        CornerRadius::same(255),
        alpha(color, 0.12),
        Stroke::new(1.0, alpha(color, 0.28)),
        StrokeKind::Inside,
    );
    p.galley(rect.center() - galley.size() / 2.0, galley, color);
    resp
}

/// Glowing progress bar using the game's accent colours.
pub fn progress_bar(ui: &mut Ui, progress: f32, a: Color32, b: Color32, height: f32) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let p = ui.painter();
    let r = CornerRadius::same((height / 2.0) as u8);
    p.rect_filled(rect, r, Color32::from_white_alpha(14));
    let progress = progress.clamp(0.0, 1.0);
    if progress > 0.0 {
        let fill = Rect::from_min_size(
            rect.min,
            Vec2::new((rect.width() * progress).max(height), height),
        );
        p.rect_filled(
            fill.expand(3.0),
            CornerRadius::same((height / 2.0 + 3.0) as u8),
            alpha(a, 0.15),
        );
        p.rect_filled(fill, r, a);
        let clip = p.with_clip_rect(fill.shrink2(Vec2::new(height / 2.0, 0.0)));
        gradient_rect(&clip, fill, a, b, true);
        p.rect_filled(
            Rect::from_min_max(
                Pos2::new(fill.right() - height, fill.top()),
                fill.right_bottom(),
            ),
            r,
            b,
        );
    }
}

/// iOS-like toggle switch.
pub fn toggle(ui: &mut Ui, on: &mut bool) -> Response {
    let size = Vec2::new(40.0, 22.0);
    let (rect, mut resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let t = ui.ctx().animate_bool_with_time(resp.id, *on, 0.15);
    let p = ui.painter();
    let bg = mix(Color32::from_rgb(34, 41, 56), BRAND, t);
    p.rect(
        rect,
        CornerRadius::same(11),
        bg,
        Stroke::new(1.0, Color32::from_white_alpha(14)),
        StrokeKind::Inside,
    );
    let x = egui::lerp(rect.left() + 11.0..=rect.right() - 11.0, t);
    p.circle_filled(
        Pos2::new(x, rect.center().y),
        8.0,
        if *on {
            Color32::from_rgb(8, 12, 18)
        } else {
            TEXT_DIM
        },
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Section heading used on all pages.
pub fn page_header(ui: &mut Ui, eyebrow: &str, title: &str, subtitle: &str) {
    ui.label(
        egui::RichText::new(eyebrow.to_uppercase())
            .font(font_heavy(10.5))
            .color(BRAND)
            .extra_letter_spacing(2.0),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(title)
            .font(font_heavy(34.0))
            .color(TEXT),
    );
    if !subtitle.is_empty() {
        ui.label(egui::RichText::new(subtitle).font(font(13.0)).color(MUTED));
    }
    ui.add_space(18.0);
}

pub fn section_label(ui: &mut Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .font(font_semi(10.5))
            .color(MUTED)
            .extra_letter_spacing(1.6),
    );
    ui.add_space(2.0);
}

/// Rounded square avatar with short name + accent gradient (used in rail and lists).
pub fn game_badge(
    ui: &mut Ui,
    size: f32,
    short: &str,
    a: Color32,
    b: Color32,
    active: bool,
    sense: Sense,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), sense);
    paint_game_badge(
        ui,
        rect,
        short,
        a,
        b,
        active,
        ui.ctx().animate_bool(resp.id, resp.hovered()),
    );
    resp
}

pub fn paint_game_badge(
    ui: &Ui,
    rect: Rect,
    short: &str,
    a: Color32,
    b: Color32,
    active: bool,
    hover: f32,
) {
    let p = ui.painter();
    let radius = CornerRadius::same((rect.width() * 0.3) as u8);
    let strength = if active { 1.0 } else { 0.35 + hover * 0.35 };
    if active {
        p.rect_filled(
            rect.expand(4.0),
            CornerRadius::same((rect.width() * 0.3 + 4.0) as u8),
            alpha(a, 0.12),
        );
    }
    p.rect_filled(
        rect,
        radius,
        mix(Color32::from_rgb(17, 22, 33), a, 0.18 * strength),
    );
    let clip = p.with_clip_rect(rect.shrink(rect.width() * 0.12));
    gradient_rect(
        &clip,
        rect,
        alpha(a, 0.55 * strength),
        alpha(b, 0.35 * strength),
        true,
    );
    p.rect_stroke(
        rect,
        radius,
        Stroke::new(
            1.0,
            alpha(mix(a, Color32::WHITE, 0.3), 0.15 + 0.35 * strength),
        ),
        StrokeKind::Inside,
    );
    let fs = if short.len() > 3 {
        rect.width() * 0.24
    } else {
        rect.width() * 0.3
    };
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        short,
        font_heavy(fs),
        mix(TEXT_DIM, Color32::WHITE, strength),
    );
}

pub fn fmt_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".into();
    }
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let i = ((bytes as f64).ln() / 1024f64.ln()).floor().min(4.0) as usize;
    let v = bytes as f64 / 1024f64.powi(i as i32);
    if i >= 3 {
        format!("{v:.1} {}", UNITS[i])
    } else {
        format!("{v:.0} {}", UNITS[i])
    }
}

pub fn fmt_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{h} h {m} min")
    } else if m > 0 {
        format!("{m} min")
    } else if secs > 0 {
        "< 1 min".into()
    } else {
        "—".into()
    }
}
