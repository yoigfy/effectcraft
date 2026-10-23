//! Touch-first navigation for narrow viewports. The desktop dock tree is left untouched.
use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use serde_json::json;

use crate::dock::PanelKind;
use crate::icons::{self, Icon};
use crate::theme::Tokens;
use crate::EffectcraftApp;

const PANEL_ID: &str = "tokeffects.mobile.active-panel";
const BREAKPOINT: f32 = 1100.0;

pub(crate) fn is_mobile(rect: Rect) -> bool {
    rect.width() < BREAKPOINT
}

pub(crate) fn active_panel(ctx: &egui::Context) -> PanelKind {
    ctx.data(|d| d.get_temp::<PanelKind>(egui::Id::new(PANEL_ID))).unwrap_or(PanelKind::Composition)
}

fn choose_panel(ctx: &egui::Context, panel: PanelKind) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(PANEL_ID), panel));
}

fn invoke(app: &mut EffectcraftApp, ctx: &egui::Context, command: &str) {
    if let Err(error) = crate::menus::invoke(app, ctx, command, json!({})) {
        app.ui.status = error;
    }
}

pub(crate) fn show_header(app: &mut EffectcraftApp, ui: &mut egui::Ui, rect: Rect, active: PanelKind) {
    let t = app.tokens;
    let painter = ui.painter().clone();
    painter.rect_filled(rect, 0.0, t.header_bg);
    painter.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.separator));

    // Load the logo once into an egui texture; do not decode or upload it on every frame.
    let ctx = ui.ctx().clone();
    let logo_id = egui::Id::new("tokeffects-mobile-app-icon");
    let logo = ctx.data(|d| d.get_temp::<egui::TextureHandle>(logo_id)).or_else(|| {
        let bytes: &[u8] = include_bytes!("../../../assets/app-icon/tokeffects.png");
        let image = image::load_from_memory(bytes).ok()?.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
        let handle = ctx.load_texture("tokeffects-mobile-app-icon", pixels, egui::TextureOptions::LINEAR);
        ctx.data_mut(|d| d.insert_temp(logo_id, handle.clone()));
        Some(handle)
    });
    let icon = Rect::from_min_size(pos2(rect.min.x + 10.0, rect.center().y - 14.0), vec2(28.0, 28.0));
    if let Some(logo) = logo {
        painter.image(logo.id(), icon, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
    painter.text(
        pos2(rect.min.x + 45.0, rect.center().y),
        Align2::LEFT_CENTER,
        "Tokeffects",
        Tokens::semibold(15.0),
        t.text,
    );

    let menu_rect = Rect::from_min_size(pos2(rect.max.x - 54.0, rect.center().y - 22.0), vec2(44.0, 44.0));
    painter.rect_filled(menu_rect, 8.0, t.row_alt);
    painter.rect_stroke(menu_rect, 8.0, Stroke::new(1.0, t.field_border), StrokeKind::Inside);
    icons::paint(&painter, menu_rect.shrink(12.0), Icon::Hamburger, t.text);
    let response = ui
        .interact(menu_rect, egui::Id::new("tokeffects-mobile-menu"), Sense::click())
        .on_hover_text("Menu");
    let open = response.clicked().then_some(egui::SetOpenCommand::Bool(true));
    egui::Popup::menu(&response).open_memory(open).show(|menu| {
        menu.set_min_width(220.0);
        if menu.button("Open project…").clicked() {
            invoke(app, menu.ctx(), "file.open");
            menu.close();
        }
        if menu.button("Import media…").clicked() {
            invoke(app, menu.ctx(), "file.import");
            menu.close();
        }
        if menu.button("Save project").clicked() {
            invoke(app, menu.ctx(), "file.save");
            menu.close();
        }
        if menu.button("Save as…").clicked() {
            invoke(app, menu.ctx(), "file.saveAs");
            menu.close();
        }
        menu.separator();
        if menu.button("Undo").clicked() {
            invoke(app, menu.ctx(), "edit.undo");
            menu.close();
        }
        if menu.button("Redo").clicked() {
            invoke(app, menu.ctx(), "edit.redo");
            menu.close();
        }
        if menu.button("Home").clicked() {
            app.ui.start_screen = true;
            menu.close();
        }
        if menu.button("Settings…").clicked() {
            crate::panels::settings::open(app, "general");
            menu.close();
        }
        menu.menu_button("Tools", |menu| {
            for slot in crate::state::Tool::SLOTS {
                menu.menu_button(slot[0].label(), |menu| {
                    for tool in slot.iter() {
                        let row = menu.selectable_label(app.ui.tool == *tool, tool.label());
                        if row.clicked() {
                            app.ui.tool = *tool;
                            menu.close();
                        }
                    }
                });
            }
        });
        menu.separator();
        menu.menu_button("Panels", |menu| {
            egui::ScrollArea::vertical().max_height(360.0).show(menu, |menu| {
                for panel in PanelKind::ALL {
                    if menu.selectable_label(active == panel, panel.title()).clicked() {
                        choose_panel(menu.ctx(), panel);
                        app.ui.start_screen = false;
                        menu.close();
                    }
                }
            });
        });
    });

    let panel_title = active.title();
    if rect.width() > 340.0 {
        let title_rect = Rect::from_min_max(
            pos2(rect.min.x + 150.0, rect.min.y),
            pos2(menu_rect.min.x - 8.0, rect.max.y),
        );
        painter.text(title_rect.left_center(), Align2::LEFT_CENTER, panel_title, Tokens::ui(11.0), t.text_dim);
    }
}

pub(crate) fn show_navigation(app: &mut EffectcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let painter = ui.painter().clone();
    painter.rect_filled(rect, 0.0, t.header_bg);
    painter.line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, t.separator));

    let choices: [(&str, Icon, PanelKind); 5] = [
        ("Project", Icon::Folder, PanelKind::Project),
        ("Viewer", Icon::Comp, PanelKind::Composition),
        ("Timeline", Icon::Graph, PanelKind::Timeline),
        ("Edit", Icon::Fx, PanelKind::Properties),
        ("Render", Icon::Play, PanelKind::RenderQueue),
    ];
    let cell = rect.width() / choices.len() as f32;
    for (index, (label, icon, panel)) in choices.iter().enumerate() {
        let r = Rect::from_min_size(pos2(rect.min.x + index as f32 * cell, rect.min.y + 2.0), vec2(cell, (rect.height() - 4.0).max(1.0)));
        let active = active_panel(ui.ctx()) == *panel;
        let response = ui.interact(r.shrink2(vec2(2.0, 2.0)), egui::Id::new(("tokeffects-mobile-tab", *panel)), Sense::click());
        if response.clicked() {
            choose_panel(ui.ctx(), *panel);
            app.ui.start_screen = false;
        }
        let colour = if active { t.text } else { t.text_faint };
        if active {
            painter.rect_filled(r.shrink2(vec2(3.0, 4.0)), 7.0, t.row_selected);
        } else if response.hovered() {
            painter.rect_filled(r.shrink2(vec2(3.0, 4.0)), 7.0, t.row_alt);
        }
        let icon_rect = Rect::from_center_size(pos2(r.center().x, r.min.y + 22.0), vec2(20.0, 20.0));
        icons::paint(&painter, icon_rect, *icon, colour);
        painter.text(pos2(r.center().x, r.max.y - 17.0), Align2::CENTER_CENTER, *label, Tokens::ui(11.5), colour);
    }
}
