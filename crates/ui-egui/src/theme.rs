//! Design tokens. Every widget reads colours/sizes from [`Tokens`] so themes apply everywhere.
//!
//! Tokeffects uses a monochrome black-and-graphite interface with no colored UI accents.
//! Fonts are Inter + JetBrains Mono (OFL).

use std::sync::Arc;

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeKind {
    #[default]
    Dark,
    Darker,
    Light,
}

impl ThemeKind {
    pub fn from_name(s: &str) -> Option<ThemeKind> {
        match s.to_ascii_lowercase().as_str() {
            "dark" | "default" => Some(ThemeKind::Dark),
            "darker" | "darkest" => Some(ThemeKind::Darker),
            "light" => Some(ThemeKind::Light),
            _ => None,
        }
    }

    /// The settings value (`appearance.theme`, `appearance.darkTheme`, `appearance.lightTheme`).
    pub fn name(self) -> &'static str {
        match self {
            ThemeKind::Dark => "dark",
            ThemeKind::Darker => "darker",
            ThemeKind::Light => "light",
        }
    }

    pub fn is_light(self) -> bool {
        self == ThemeKind::Light
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub kind: ThemeKind,
    pub app_bg: Color32,
    pub header_bg: Color32,
    pub panel_bg: Color32,
    pub tab_text: Color32,
    pub tab_text_active: Color32,
    pub focus: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_faint: Color32,
    pub icon: Color32,
    pub icon_active: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub field_bg: Color32,
    pub field_border: Color32,
    pub separator: Color32,
    pub row: Color32,
    pub row_alt: Color32,
    pub row_selected: Color32,
    pub hot_text: Color32,
    pub tl_bg: Color32,
    pub tl_ruler_bg: Color32,
    pub tl_ruler_tick: Color32,
    pub tl_ruler_text: Color32,
    pub cti: Color32,
    pub work_area: Color32,
    pub cache_green: Color32,
    /// Timeline cache bar for frames held in the disk cache (not in RAM).
    pub cache_blue: Color32,
    pub keyframe: Color32,
    pub keyframe_selected: Color32,
    pub pasteboard: Color32,
    pub timecode: Color32,
    pub danger: Color32,
    pub warning: Color32,
    /// Effect Controls effect headers, and the header of the effect being worked on (it or one
    /// of its properties selected).
    pub fx_header: Color32,
    pub fx_header_active: Color32,
    /// Timeline: a selected layer's row in the time graph, the boxes behind the A/V and layer
    /// switches, the time navigator and the work area bar.
    pub tl_graph_selected: Color32,
    pub switch_well: Color32,
    pub tl_nav_bar: Color32,
    pub tl_work_area_bar: Color32,
    /// The Timeline's expression field and the colour of enabled expressions (their text and
    /// the names of properties that have one).
    pub expr_bg: Color32,
    pub expr_text: Color32,
    /// Label colours (`Label::ALL` order), from Settings ▸ Labels.
    pub labels: [Color32; 17],
    pub radius: f32,
    pub radius_sm: f32,
    pub gap: f32,
    pub tab_h: f32,
    pub row_h: f32,
    /// Settings ▸ Appearance ▸ Use Gradients: panel tab strips and buttons get a soft vertical
    /// gradient (off: flat fills).
    pub gradients: bool,
}

/// A vertical gradient fill (`top` → `bottom`) over `rect`.
pub fn gradient_rect(painter: &egui::Painter, rect: egui::Rect, top: Color32, bottom: Color32) {
    let mut m = egui::Mesh::default();
    m.colored_vertex(rect.left_top(), top);
    m.colored_vertex(rect.right_top(), top);
    m.colored_vertex(rect.right_bottom(), bottom);
    m.colored_vertex(rect.left_bottom(), bottom);
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(m));
}

impl Tokens {
    /// The gradient top colour for a fill (a little lighter), or the fill itself when gradients
    /// are off.
    pub fn grad_top(&self, c: Color32) -> Color32 {
        if self.gradients { c.lerp_to_gamma(Color32::WHITE, 0.06) } else { c }
    }
}

impl Tokens {
    pub fn for_kind(kind: ThemeKind) -> Self {
        let dark = Tokens {
            kind,
            app_bg: Color32::from_rgb(0x00, 0x00, 0x00),
            header_bg: Color32::from_rgb(0x12, 0x12, 0x12),
            panel_bg: Color32::from_rgb(0x1c, 0x1c, 0x1c),
            tab_text: Color32::from_gray(154),
            tab_text_active: Color32::from_gray(245),
            focus: Color32::from_gray(184),
            accent: Color32::from_gray(188),
            accent_hover: Color32::from_gray(220),
            text: Color32::from_gray(232),
            text_dim: Color32::from_gray(174),
            text_faint: Color32::from_gray(122),
            icon: Color32::from_gray(188),
            icon_active: Color32::from_rgb(0xff, 0xff, 0xff),
            hover: Color32::from_gray(48),
            pressed: Color32::from_gray(66),
            field_bg: Color32::from_gray(8),
            field_border: Color32::from_gray(68),
            separator: Color32::from_gray(39),
            row: Color32::from_gray(22),
            row_alt: Color32::from_gray(30),
            row_selected: Color32::from_gray(66),
            hot_text: Color32::from_gray(220),
            tl_bg: Color32::from_gray(8),
            tl_ruler_bg: Color32::from_gray(24),
            tl_ruler_tick: Color32::from_gray(125),
            tl_ruler_text: Color32::from_gray(182),
            cti: Color32::from_gray(238),
            work_area: Color32::from_gray(88),
            cache_green: Color32::from_gray(156),
            cache_blue: Color32::from_gray(112),
            keyframe: Color32::from_gray(164),
            keyframe_selected: Color32::from_gray(238),
            pasteboard: Color32::from_gray(2),
            timecode: Color32::from_gray(232),
            danger: Color32::from_gray(204),
            warning: Color32::from_gray(154),
            fx_header: Color32::from_gray(34),
            fx_header_active: Color32::from_gray(64),
            tl_graph_selected: Color32::from_gray(54),
            switch_well: Color32::from_gray(17),
            tl_nav_bar: Color32::from_gray(88),
            tl_work_area_bar: Color32::from_gray(122),
            expr_bg: Color32::from_gray(8),
            expr_text: Color32::from_gray(222),
            labels: default_labels(),
            radius: 8.0,
            radius_sm: 4.0,
            gap: 4.0,
            tab_h: 32.0,
            row_h: 20.0,
            gradients: false,
        };
        match kind {
            ThemeKind::Dark => dark,
            ThemeKind::Darker => Tokens {
                app_bg: Color32::from_rgb(0x08, 0x08, 0x08),
                header_bg: Color32::from_rgb(0x16, 0x16, 0x16),
                panel_bg: Color32::from_rgb(0x19, 0x19, 0x19),
                row: Color32::from_rgb(0x19, 0x19, 0x19),
                row_alt: Color32::from_rgb(0x1d, 0x1d, 0x1d),
                tl_bg: Color32::from_rgb(0x12, 0x12, 0x12),
                tl_ruler_bg: Color32::from_rgb(0x19, 0x19, 0x19),
                field_bg: Color32::from_rgb(0x0e, 0x0e, 0x0e),
                pasteboard: Color32::from_rgb(0x11, 0x11, 0x11),
                ..dark
            },
            ThemeKind::Light => Tokens {
                app_bg: Color32::from_gray(242),
                header_bg: Color32::from_gray(250),
                panel_bg: Color32::WHITE,
                tab_text: Color32::from_gray(116),
                tab_text_active: Color32::from_gray(27),
                text: Color32::from_gray(42),
                text_dim: Color32::from_gray(100),
                text_faint: Color32::from_gray(116),
                hot_text: Color32::from_gray(40),
                timecode: Color32::from_gray(40),
                focus: Color32::from_gray(170),
                accent: Color32::from_gray(124),
                accent_hover: Color32::from_gray(108),
                cti: Color32::from_gray(124),
                keyframe_selected: Color32::from_gray(124),
                icon: Color32::from_gray(86),
                icon_active: Color32::from_gray(27),
                hover: Color32::from_gray(242),
                pressed: Color32::from_gray(231),
                field_bg: Color32::WHITE,
                field_border: Color32::from_gray(210),
                separator: Color32::from_gray(227),
                row: Color32::WHITE,
                row_alt: Color32::from_gray(248),
                row_selected: Color32::from_gray(236),
                tl_bg: Color32::from_gray(239),
                tl_ruler_bg: Color32::from_gray(246),
                tl_ruler_text: Color32::from_gray(87),
                pasteboard: Color32::from_gray(231),
                keyframe: Color32::from_gray(118),
                fx_header: Color32::from_gray(245),
                fx_header_active: Color32::from_gray(236),
                tl_graph_selected: Color32::from_gray(229),
                switch_well: Color32::from_gray(235),
                tl_nav_bar: Color32::from_gray(176),
                tl_work_area_bar: Color32::from_gray(188),
                expr_bg: Color32::from_gray(250),
                expr_text: Color32::from_gray(82),
                ..dark
            },
        }
    }

    pub fn mono(size: f32) -> FontId {
        FontId::new(size, FontFamily::Monospace)
    }
    pub fn ui(size: f32) -> FontId {
        FontId::new(size, FontFamily::Proportional)
    }
    pub fn semibold(size: f32) -> FontId {
        FontId::new(size, FontFamily::Name("semibold".into()))
    }
    pub fn medium(size: f32) -> FontId {
        FontId::new(size, FontFamily::Name("medium".into()))
    }
    /// sRGB colour of an item/layer label (Settings ▸ Labels).
    pub fn label(&self, l: effectcraft_color::Label) -> Color32 {
        let i = effectcraft_color::Label::ALL.iter().position(|x| *x == l).unwrap_or(0);
        self.labels[i]
    }

    /// Theme tokens for the current settings: theme, UI brightness and label colours.
    /// The tokens for theme `kind` (resolved from the appearance mode) with the rest of the
    /// Appearance settings: brightness, label colours, gradients.
    pub fn from_prefs(p: &effectcraft_engine::prefs::Prefs, kind: ThemeKind) -> Tokens {
        let mut t = Tokens::for_kind(kind).with_brightness(p.appearance.brightness as f32);
        for (i, l) in effectcraft_color::Label::ALL.iter().enumerate() {
            let [r, g, b] = p.label_rgb(*l);
            // Keep user-defined label distinctions by luminance, never hue.
            let gray = ((u16::from(r) + u16::from(g) + u16::from(b)) / 3) as u8;
            t.labels[i] = Color32::from_gray(gray);
        }
        t.gradients = p.appearance.use_gradients;
        t
    }

    /// Lighten (b > 0) or darken (b < 0) the interface greys. The entire UI palette is monochrome;
    /// saved label colours are converted to greyscale when the theme tokens are built.
    pub fn with_brightness(mut self, b: f32) -> Tokens {
        let b = b.clamp(-1.0, 1.0);
        if b.abs() < 1e-3 {
            return self;
        }
        let adj = |c: &mut Color32| {
            let f = |v: u8| -> u8 {
                let v = v as f32;
                (if b > 0.0 { v + (110.0 - v).max(0.0) * b * 0.8 + 10.0 * b } else { v * (1.0 + b * 0.6) }).round().clamp(0.0, 255.0) as u8
            };
            *c = Color32::from_rgb(f(c.r()), f(c.g()), f(c.b()));
        };
        for c in [
            &mut self.app_bg,
            &mut self.header_bg,
            &mut self.panel_bg,
            &mut self.hover,
            &mut self.pressed,
            &mut self.field_bg,
            &mut self.field_border,
            &mut self.separator,
            &mut self.row,
            &mut self.row_alt,
            &mut self.row_selected,
            &mut self.tl_bg,
            &mut self.tl_ruler_bg,
            &mut self.pasteboard,
            &mut self.work_area,
            &mut self.fx_header,
            &mut self.tl_graph_selected,
            &mut self.switch_well,
            &mut self.expr_bg,
        ] {
            adj(c);
        }
        self
    }
}

fn default_labels() -> [Color32; 17] {
    // Shades still distinguish layers while keeping the interface monochrome.
    [42u8, 52, 62, 72, 82, 92, 102, 112, 122, 132, 142, 152, 162, 176, 190, 207, 226]
        .map(Color32::from_gray)
}

static INTER_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
static INTER_MEDIUM: &[u8] = include_bytes!("../../../assets/fonts/Inter-Medium.ttf");
static INTER_SEMIBOLD: &[u8] = include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf");
static JETBRAINS_MONO: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf");

/// Install fonts (Inter, Inter Medium/SemiBold, JetBrains Mono) and egui visuals. `language` is the
/// resolved interface language from [`crate::i18n::language`] (`en`, `ja`, `zh-hans` or `zh-hant`),
/// not the stored `general.language` preference: `system` is already resolved there, so a Chinese
/// system draws Chinese with the default setting. The CJK fallback that draws the interface's own
/// script is registered first, so a Chinese interface is drawn by a Chinese face and a Japanese one
/// by a Japanese face. `install` is called again when the language changes (see the `EffectcraftApp`
/// frame loop), so the fonts do not need a restart.
pub fn install(ctx: &egui::Context, t: &Tokens, language: &str) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert("inter".into(), Arc::new(FontData::from_static(INTER_REGULAR)));
    fonts.font_data.insert("inter-medium".into(), Arc::new(FontData::from_static(INTER_MEDIUM)));
    fonts.font_data.insert("inter-semibold".into(), Arc::new(FontData::from_static(INTER_SEMIBOLD)));
    fonts.font_data.insert("jbmono".into(), Arc::new(FontData::from_static(JETBRAINS_MONO)));
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "inter".into());
    fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "jbmono".into());
    fonts.families.insert(FontFamily::Name("semibold".into()), vec!["inter-semibold".into(), "inter".into()]);
    fonts.families.insert(FontFamily::Name("medium".into()), vec!["inter-medium".into(), "inter".into()]);
    // Inter and egui's own fallbacks have no small triangles (the `▸` of "File ▸ Import"), which
    // would draw as missing-glyph boxes: JetBrains Mono fills in the symbols the others lack.
    for (family, stack) in &mut fonts.families {
        if *family != FontFamily::Monospace {
            stack.push("jbmono".into());
        }
    }
    install_script_fallbacks(&mut fonts, language);
    ctx.set_fonts(fonts);
    apply_visuals(ctx, t);
}

/// Scripts the bundled fonts lack beyond CJK, each probed with one of its letters: names typed in
/// them (layers, items, comments) are drawn by an installed face instead of missing-glyph boxes
/// (#394). egui shapes each run of one face, so Arabic letters join and a word reads right to
/// left; it has no bidirectional layout, so the words of a name still follow each other left to
/// right.
#[cfg(not(target_arch = "wasm32"))]
const SCRIPT_PROBES: &[(char, &str)] = &[
    ('م', "arabic-system"),
    ('א', "hebrew-system"),
    ('क', "devanagari-system"),
    ('ক', "bengali-system"),
    ('ਕ', "gurmukhi-system"),
    ('ક', "gujarati-system"),
    ('க', "tamil-system"),
    ('క', "telugu-system"),
    ('ಕ', "kannada-system"),
    ('ക', "malayalam-system"),
    ('ก', "thai-system"),
];

/// Register the system fallbacks for scripts the bundled fonts lack in every font family, and
/// return their names in registration order (empty when nothing installed covers a probe, so a
/// machine without those fonts simply keeps the bundled ones). A face that covers several probes
/// (Segoe UI draws Arabic and Hebrew, Nirmala UI every Indic script) is registered once.
///
/// CJK comes first. The text engine has its own script-aware fallback (#84), but which face it
/// picks for Han ideographs depends on the locale environment: a Chinese interface on a
/// Japanese-locale machine would be drawn with the Japanese face. The interface language is the
/// better signal, so the probe for its script comes first — kana for Japanese, a Han character for
/// the Chinese catalogs. Both scripts are registered, so file, layer and template names in the
/// other one still render; then [`SCRIPT_PROBES`], then Hangul, since the Japanese and Chinese faces
/// may lack it (Hiragino Sans on macOS has none). The web build has no system fonts
/// and installs nothing.
fn install_script_fallbacks(family_fonts: &mut FontDefinitions, language: &str) -> Vec<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (family_fonts, language);
        Vec::new()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use effectcraft_text::fonts;
        // Kana is unambiguously Japanese; Han ideographs are shared, so the order decides which
        // language's face draws them.
        const KANA: (char, &str) = ('あ', "japanese-system");
        // Hangul is probed last: with no Korean font installed the lookup can land on a catch-all
        // face (Unifont) that also covers Arabic or Indic text but can't shape it, so the
        // script-specific faces must be registered ahead of it. A face already seen is skipped.
        const HANGUL: (char, &str) = ('한', "korean-system");
        // Han text follows the interface language, not the operating system's locale.
        fonts::set_cjk_locale(cjk_locale(language));
        let base = fonts::resolve("Inter", "Regular").face;
        let han = (han_probe(language), "chinese-system");
        let cjk = if is_chinese(language) { [han, KANA] } else { [KANA, han] };
        let mut names = Vec::new();
        let mut seen = Vec::new();
        for &(probe, name) in cjk.iter().chain(SCRIPT_PROBES).chain(std::iter::once(&HANGUL)) {
            let id = fonts::fallback_for(probe, base);
            if seen.contains(&id) {
                continue;
            }
            let face = fonts::face(id);
            if !face.has_char(probe) {
                continue; // nothing installed covers this script
            }
            let Some(font) = face.font() else { continue };
            seen.push(id);
            // Use the already-read, parsed bytes rather than reading a font file twice.
            let mut data = FontData::from_owned(font.data().as_bytes().to_vec());
            data.index = face.info.index;
            family_fonts.font_data.insert(name.to_string(), Arc::new(data));
            for family in family_fonts.families.values_mut() {
                family.push(name.to_string());
            }
            names.push(name.to_string());
        }
        names
    }
}

/// The CJK locale the text engine should prefer for Han text: the interface language when it names
/// one, otherwise the empty string, which leaves the operating system's locale in charge.
#[cfg(not(target_arch = "wasm32"))]
fn cjk_locale(language: &str) -> &str {
    match language {
        "ja" | "zh-hans" | "zh-hant" => language,
        _ => "",
    }
}

/// The Han character that picks the Chinese fallback face. A Simplified Chinese interface asks for
/// a simplified-only hanzi: a Japanese face covers the shared ideographs (文) but not 页, 选 or 删,
/// and the fallback takes a face already loaded first, so the Japanese face loaded for kana would
/// draw the Chinese interface with boxes (#1141).
#[cfg(not(target_arch = "wasm32"))]
fn han_probe(language: &str) -> char {
    if language == "zh-hans" { '页' } else { '文' }
}

/// Whether `language` is one of the Chinese catalogs.
#[cfg(not(target_arch = "wasm32"))]
fn is_chinese(language: &str) -> bool {
    matches!(language, "zh-hans" | "zh-hant")
}

/// egui's theme preference for an appearance mode showing a `light` (or dark) theme. Sync with
/// System leaves it at System: a concrete preference makes egui set the native window's
/// appearance, and on macOS a window with its own appearance no longer hears when the system's
/// changes (#642). A fixed Light or Dark mode pins the window to match.
pub fn theme_preference(appearance_mode: &str, light: bool) -> egui::ThemePreference {
    match appearance_mode {
        "auto" => egui::ThemePreference::System,
        _ if light => egui::ThemePreference::Light,
        _ => egui::ThemePreference::Dark,
    }
}

pub fn apply_visuals(ctx: &egui::Context, t: &Tokens) {
    let light = t.kind.is_light();
    let mut v = if light { Visuals::light() } else { Visuals::dark() };
    v.panel_fill = t.panel_bg;
    v.window_fill = t.panel_bg;
    v.extreme_bg_color = t.field_bg;
    v.faint_bg_color = t.row_alt;
    v.override_text_color = Some(t.text);
    v.selection.bg_fill = t.accent;
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.hyperlink_color = t.accent;
    v.window_stroke = Stroke::new(1.0, t.field_border);
    v.window_corner_radius = egui::CornerRadius::same(8);
    v.menu_corner_radius = egui::CornerRadius::same(6);
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 6], blur: 12, spread: 0, color: Color32::from_black_alpha(150) };
    v.window_shadow = egui::epaint::Shadow { offset: [0, 10], blur: 20, spread: 0, color: Color32::from_black_alpha(170) };
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = egui::CornerRadius::same(t.radius_sm as u8);
    }
    v.widgets.noninteractive.bg_fill = t.panel_bg;
    v.widgets.noninteractive.weak_bg_fill = t.panel_bg;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.separator);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.inactive.bg_fill = t.field_bg;
    v.widgets.inactive.weak_bg_fill = t.field_bg;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.field_border);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.hovered.bg_fill = t.hover;
    v.widgets.hovered.weak_bg_fill = t.hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.field_border);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, t.tab_text_active);
    v.widgets.active.bg_fill = t.pressed;
    v.widgets.active.weak_bg_fill = t.pressed;
    v.widgets.active.fg_stroke = Stroke::new(1.0, t.tab_text_active);
    v.widgets.open.bg_fill = t.hover;
    v.widgets.open.weak_bg_fill = t.hover;
    // egui keeps one style per light/dark theme and picks it by its theme preference (see
    // [`theme_preference`]); both get ours, so whichever it picks, egui's defaults never show.
    ctx.all_styles_mut(|s| {
        s.visuals = v.clone();
        s.spacing.item_spacing = egui::vec2(7.0, 5.0);
        s.spacing.button_padding = egui::vec2(9.0, 4.0);
        s.spacing.interact_size.y = 24.0;
        s.spacing.menu_margin = egui::Margin::same(7);
        // Scroll bars always show (egui's float in only over the list), so long menus and lists
        // can be dragged without a mouse wheel or touchpad (#269).
        s.spacing.scroll = egui::style::ScrollStyle::solid();
        // The handle takes the text colour: the fill it would otherwise get is the colour of its
        // track (both are the field background), which left it invisible until hovered.
        s.spacing.scroll.foreground_color = true;
        s.text_styles.insert(TextStyle::Body, FontId::new(12.5, FontFamily::Proportional));
        s.text_styles.insert(TextStyle::Button, FontId::new(12.5, FontFamily::Proportional));
        s.text_styles.insert(TextStyle::Small, FontId::new(11.0, FontFamily::Proportional));
        s.text_styles.insert(TextStyle::Heading, FontId::new(16.0, FontFamily::Name("semibold".into())));
        s.text_styles.insert(TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace));
        s.animation_time = 0.06;
    });
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod script_font_tests {
    use effectcraft_text::fonts;

    /// Whether an installed face covers `probe` (else the glyph check is skipped).
    fn installed(probe: char) -> bool {
        let base = fonts::resolve("Inter", "Regular").face;
        fonts::face(fonts::fallback_for(probe, base)).has_char(probe)
    }

    /// Every character of `text` draws a real glyph in every UI font family with the `language`
    /// interface's fonts installed.
    fn assert_drawn(language: &str, text: &str) {
        let ctx = egui::Context::default();
        super::install(&ctx, &super::Tokens::for_kind(super::ThemeKind::Dark), language);
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        // No renderer here: drop the frame's texture uploads (egui asserts on unhandled ones in debug).
        out.textures_delta.clear();
        ctx.fonts_mut(|fonts| {
            for family in [
                egui::FontFamily::Proportional,
                egui::FontFamily::Monospace,
                egui::FontFamily::Name("medium".into()),
                egui::FontFamily::Name("semibold".into()),
            ] {
                let font = egui::FontId::new(13.0, family);
                // epaint 0.36's has_glyph compares face keys, so it reports a false
                // negative when a real glyph shares the replacement face.
                // Check the rendered atlas glyph instead of that face-level predicate.
                let missing = fonts.layout_no_wrap("\u{10ffff}".into(), font.clone(), egui::Color32::WHITE);
                let missing_uv = missing.rows[0].glyphs[0].uv_rect;
                for ch in text.chars() {
                    let rendered = fonts.layout_no_wrap(ch.to_string(), font.clone(), egui::Color32::WHITE);
                    let uv = rendered.rows[0].glyphs[0].uv_rect;
                    assert!(uv.size.x > 0.0 && uv.size.y > 0.0, "empty {ch} in {font:?}");
                    assert_ne!((uv.min, uv.max), (missing_uv.min, missing_uv.max), "replacement glyph for {ch} in {font:?}");
                }
            }
        });
    }

    #[test]
    fn installed_japanese_fallback_is_available_in_all_ui_families() {
        if !installed('あ') {
            eprintln!("no Japanese system font installed; skipping glyph coverage");
            return;
        }
        assert_drawn("ja", "日本語コンポジションレイヤーエフェクト設定");
    }

    /// #394: Arabic, Hebrew, Indic and Thai names are drawn by an installed face in an English
    /// interface: the UI fonts were Inter and the CJK fallbacks only, so a layer named in Arabic
    /// showed as missing-glyph boxes.
    #[test]
    fn installed_script_fallbacks_draw_arabic_hebrew_indic_and_thai_names() {
        // Letters only: a vowel sign or virama on its own is drawn on a dotted circle.
        for text in ["طبقةمرحبا", "שכבה", "परत", "কলম", "ਕਲਮ", "કલમ", "கடல", "కలమ", "ಕಲಮ", "കലമ", "กขค"]
        {
            let Some(probe) = text.chars().next() else { continue };
            if !installed(probe) {
                eprintln!("no system font covers {text}; skipping its glyph coverage");
                continue;
            }
            assert_drawn("en", text);
        }
    }

    /// #1141: switching the interface to Simplified Chinese drew most labels as boxes. The English
    /// interface had already loaded a Japanese face for kana, and that face covers the Han probe,
    /// so it became the Chinese fallback too, without the simplified-only hanzi.
    #[test]
    fn simplified_chinese_interface_draws_simplified_hanzi_after_another_language() {
        let ctx = egui::Context::default();
        let tokens = super::Tokens::for_kind(super::ThemeKind::Dark);
        super::install(&ctx, &tokens, "en");
        super::install(&ctx, &tokens, "zh-hans");
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        let font = egui::FontId::new(13.0, egui::FontFamily::Proportional);
        let missing: String = ctx.fonts_mut(|fonts| {
            let replacement = fonts.layout_no_wrap("\u{10ffff}".into(), font.clone(), egui::Color32::WHITE).rows[0].glyphs[0].uv_rect;
            // Simplified-only hanzi of the zh-hans catalog (选择, 删除, 图层, 设置, 导入, 页面).
            "选删层设导页"
                .chars()
                .filter(|ch| {
                    let uv = fonts.layout_no_wrap(ch.to_string(), font.clone(), egui::Color32::WHITE).rows[0].glyphs[0].uv_rect;
                    (uv.min, uv.max) == (replacement.min, replacement.max)
                })
                .collect()
        });
        // Asked after the layout: the coverage lookup itself loads a face with these hanzi.
        let drawable: String = missing.chars().filter(|&ch| installed(ch)).collect();
        assert!(drawable.is_empty(), "an installed face covers {drawable}, but the Simplified Chinese interface draws boxes");
    }

    /// egui shapes each run of one face: Arabic letters take their joined forms, and an Arabic
    /// word reads right to left (its first letter drawn rightmost).
    #[test]
    fn arabic_words_are_joined_and_read_right_to_left() {
        if !installed('م') {
            eprintln!("no Arabic system font installed; skipping");
            return;
        }
        let ctx = egui::Context::default();
        super::install(&ctx, &super::Tokens::for_kind(super::ThemeKind::Dark), "en");
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        ctx.fonts_mut(|fonts| {
            let font = egui::FontId::new(13.0, egui::FontFamily::Proportional);
            let missing = fonts.layout_no_wrap("\u{10ffff}".into(), font.clone(), egui::Color32::WHITE).rows[0].glyphs[0].uv_rect;
            let alone = fonts.layout_no_wrap("م".into(), font.clone(), egui::Color32::WHITE);
            let word = fonts.layout_no_wrap("مرحبا".into(), font, egui::Color32::WHITE);
            let glyphs = &word.rows[0].glyphs;
            let meem = glyphs.iter().find(|g| g.chr == 'م').expect("the meem is laid out");
            let alif = glyphs.iter().find(|g| g.chr == 'ا').expect("the alif is laid out");
            assert!(meem.pos.x > alif.pos.x, "the first letter is drawn rightmost: {} vs {}", meem.pos.x, alif.pos.x);
            let isolated = alone.rows[0].glyphs[0].uv_rect;
            for uv in [meem.uv_rect, isolated] {
                assert_ne!((uv.min, uv.max), (missing.min, missing.max), "a real glyph, not the replacement");
            }
            assert_ne!((meem.uv_rect.min, meem.uv_rect.max), (isolated.min, isolated.max), "the meem takes its initial form");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The last `n` font names of `family`, in order.
    fn tail(family: &[String], n: usize) -> Vec<&str> {
        family.iter().rev().take(n).rev().map(String::as_str).collect()
    }

    #[test]
    fn both_themes_use_only_grayscale_ui_tokens() {
        for kind in [ThemeKind::Dark, ThemeKind::Darker, ThemeKind::Light] {
            let t = Tokens::for_kind(kind);
            for color in [
                t.app_bg, t.header_bg, t.panel_bg, t.tab_text, t.tab_text_active, t.focus,
                t.accent, t.accent_hover, t.text, t.text_dim, t.text_faint, t.icon,
                t.icon_active, t.hover, t.pressed, t.field_bg, t.field_border, t.separator,
                t.row, t.row_alt, t.row_selected, t.hot_text, t.tl_bg, t.tl_ruler_bg,
                t.tl_ruler_tick, t.tl_ruler_text, t.cti, t.work_area, t.cache_green,
                t.cache_blue, t.keyframe, t.keyframe_selected, t.pasteboard, t.timecode,
                t.danger, t.warning, t.fx_header, t.fx_header_active, t.tl_graph_selected,
                t.switch_well, t.tl_nav_bar, t.tl_work_area_bar, t.expr_bg, t.expr_text,
            ] {
                assert_eq!(color.r(), color.g(), "{kind:?} token is not grayscale: {color:?}");
                assert_eq!(color.g(), color.b(), "{kind:?} token is not grayscale: {color:?}");
            }
            for color in t.labels {
                assert_eq!(color.r(), color.g(), "{kind:?} label is not grayscale: {color:?}");
                assert_eq!(color.g(), color.b(), "{kind:?} label is not grayscale: {color:?}");
            }
        }
    }

    #[test]
    fn cjk_fallbacks_follow_the_interface_language() {
        // A Chinese interface asks for the Han probe first, so Han text is drawn by a Chinese face
        // even when the operating system's locale is not Chinese.
        assert_eq!(cjk_locale("zh-hans"), "zh-hans");
        assert_eq!(cjk_locale("zh-hant"), "zh-hant");
        assert_eq!(cjk_locale("ja"), "ja");
        assert_eq!(cjk_locale("en"), "");
        assert_eq!(cjk_locale("system"), "");
        let mut chinese = FontDefinitions::default();
        let installed = install_script_fallbacks(&mut chinese, "zh-hans");
        let cjk = |names: &[String]| names.iter().filter(|n| matches!(n.as_str(), "chinese-system" | "japanese-system")).cloned().collect::<Vec<_>>();
        if cjk(&installed).is_empty() {
            eprintln!("no CJK font is installed: only the language plumbing can be checked here");
        } else {
            assert_eq!(installed.first().map(String::as_str), Some("chinese-system"), "the Han probe comes first");
            assert!(chinese.font_data.contains_key("chinese-system"));
        }
        let expected: Vec<&str> = installed.iter().map(String::as_str).collect();
        for (family, stack) in &chinese.families {
            assert_eq!(tail(stack, installed.len()), expected, "{family:?}");
        }
        // A Japanese interface keeps the kana probe first, as it was before the language existed.
        let mut japanese = FontDefinitions::default();
        let installed = install_script_fallbacks(&mut japanese, "ja");
        if let Some(first) = cjk(&installed).first() {
            assert_eq!(installed.first(), Some(first));
            assert_eq!(first, "japanese-system", "the kana probe comes first");
        }
        // And an English interface leaves the operating system's locale in charge.
        let mut english = FontDefinitions::default();
        install_script_fallbacks(&mut english, "en");
    }

    #[test]
    fn every_family_reaches_the_symbol_fallback() {
        let ctx = egui::Context::default();
        install(&ctx, &Tokens::for_kind(Default::default()), "en");
        ctx.run_ui(Default::default(), |_| {}).textures_delta.clear();
        for family in [FontFamily::Proportional, FontFamily::Monospace, FontFamily::Name("medium".into()), FontFamily::Name("semibold".into())] {
            let id = FontId::new(12.0, family);
            // Not has_glyph: it compares face keys, so Inter's `•` reads as missing when no face in
            // the stack has egui's `◻` and Inter's `?` becomes the replacement, as happens once
            // another test has loaded a system face without `◻` (the fallback tries loaded faces first).
            let uv = |s: String| ctx.fonts_mut(|f| f.layout_no_wrap(s, id.clone(), egui::Color32::WHITE).rows[0].glyphs[0].uv_rect);
            let missing = uv("\u{10ffff}".into());
            for c in "▸•›·▶◀●○⚠…×".chars() {
                let drawn = uv(c.to_string());
                assert_ne!((drawn.min, drawn.max), (missing.min, missing.max), "{c} has no glyph in {id:?}");
            }
        }
    }

    #[test]
    fn hangul_gets_a_face_when_one_is_installed() {
        use effectcraft_text::fonts;
        let mut fonts_def = FontDefinitions::default();
        install_script_fallbacks(&mut fonts_def, "en");
        let base = fonts::resolve("Inter", "Regular").face;
        let id = fonts::fallback_for('한', base);
        let face = fonts::face(id);
        if !face.has_char('한') {
            eprintln!("no Hangul font is installed: nothing to check here");
            return;
        }
        let Some(font) = face.font() else { return };
        let bytes = font.data().as_bytes();
        let covered = fonts_def.font_data.values().any(|d| d.font.as_ref() == bytes && d.index == face.info.index);
        assert!(covered, "a face covering Hangul must be registered with egui");
    }

    #[test]
    fn a_fallback_face_is_registered_at_most_once() {
        let mut fonts = FontDefinitions::default();
        let installed = install_script_fallbacks(&mut fonts, "zh-hant");
        let mut unique = installed.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), installed.len(), "no name may be registered twice: {installed:?}");
        let faces: Vec<_> = installed
            .iter()
            .map(|name| {
                let data = fonts.font_data.get(name).unwrap_or_else(|| panic!("{name} is missing its font data"));
                (data.font.as_ref(), data.index)
            })
            .collect();
        for (i, a) in faces.iter().enumerate() {
            assert!(!faces[i + 1..].contains(a), "{} shares its face with a later fallback: {installed:?}", installed[i]);
        }
        // Leave the operating system's locale behind for whatever runs next.
        install_script_fallbacks(&mut fonts, "system");
    }
}
