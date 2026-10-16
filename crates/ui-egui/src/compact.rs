//! Compact layout for phone-sized windows (the web app on a phone, a narrow desktop window).
//!
//! Compact is not a setting: it follows the window's width every frame. The side panels and the
//! rail collapse into drawers that open over the page, the bars keep to one row of icons, and
//! controls grow to touch size. Wider windows keep the desktop layout unchanged.

use egui::{Align, Color32, Layout, Rect, Sense, pos2, vec2};

use crate::theme::Tokens;
use crate::{PdfCraftApp, RightPanel, icons};

/// Windows narrower than this (in logical points) use the compact layout.
pub const COMPACT_WIDTH: f32 = 700.0;
/// The side of a touch-sized button.
pub const TOUCH: f32 = 40.0;
/// The widest a drawer gets, and the strip of page it always leaves uncovered to tap it closed.
const DRAWER_MAX: f32 = 360.0;
const DRAWER_GAP: f32 = 48.0;

/// The panel a compact window shows as an overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drawer {
    /// The tools panel (the desktop's left panel).
    Tools,
    /// The navigation panels (the desktop's rail and right panel).
    Panels,
}

/// Whether a window `width` points wide uses the compact layout. A width that is not a positive
/// number (no window yet) keeps the desktop layout.
pub fn is_compact(width: f32) -> bool {
    width.is_finite() && width > 0.0 && width < COMPACT_WIDTH
}

/// A drawer's width in an area `available` points wide.
pub fn drawer_width(available: f32) -> f32 {
    if !available.is_finite() {
        return 0.0;
    }
    (available - DRAWER_GAP).clamp(0.0, DRAWER_MAX)
}

/// Touch-sized spacing for every widget drawn in a compact window.
pub fn touch_style(style: &mut egui::Style) {
    let s = &mut style.spacing;
    s.interact_size.y = s.interact_size.y.max(32.0);
    s.button_padding = vec2(s.button_padding.x.max(10.0), s.button_padding.y.max(7.0));
    s.item_spacing.y = s.item_spacing.y.max(6.0);
}

/// The navigation panels the Panels drawer switches between (the desktop rail's buttons).
const PANELS: [(RightPanel, &str, &str); 8] = [
    (RightPanel::Pages, "files", "Page thumbnails"),
    (RightPanel::Bookmarks, "bookmark", "Bookmarks"),
    (RightPanel::Comments, "message-square-text", "Comments"),
    (RightPanel::Fields, "text-cursor-input", "Form fields"),
    (RightPanel::Layers, "layers", "Layers"),
    (RightPanel::Attachments, "paperclip", "Attachments"),
    (RightPanel::Signatures, "signature", "Signatures"),
    (RightPanel::Tags, "tag", "Tags"),
];

/// Draws the open drawer over `area` (the window below the bars), with a scrim over the rest of
/// the page that closes it when tapped.
pub fn show_drawer(app: &mut PdfCraftApp, ctx: &egui::Context, area: Rect) {
    let Some(drawer) = app.compact_drawer else { return };
    if drawer == Drawer::Panels && app.active.is_none() {
        app.compact_drawer = None;
        return;
    }
    let t = Tokens::get(ctx);
    let width = drawer_width(area.width());
    let panel = match drawer {
        Drawer::Tools => Rect::from_min_size(area.min, vec2(width, area.height())),
        Drawer::Panels => Rect::from_min_max(pos2(area.right() - width, area.top()), area.max),
    };
    let mut close = false;
    egui::Area::new(egui::Id::new("compact-drawer")).order(egui::Order::Middle).fixed_pos(area.min).constrain(false).show(ctx, |ui| {
        ui.set_clip_rect(area);
        let scrim = ui.allocate_rect(area, Sense::click());
        ui.painter().rect_filled(area, 0.0, Color32::from_black_alpha(96));
        ui.painter().rect_filled(panel, 0.0, t.panel);
        let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(panel).layout(Layout::top_down(Align::Min)));
        ui.set_clip_rect(panel);
        // The tools panel has its own close button; the navigation panels get a row that
        // switches between them, as the desktop's rail does.
        if drawer == Drawer::Panels {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                egui::ScrollArea::horizontal().id_salt("compact-panels").max_width(panel.width() - TOUCH - 4.0).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (kind, icon, tip) in PANELS {
                            if icons::button(ui, icon, TOUCH, app.right == Some(kind), tl!(tip)).clicked() {
                                app.choose_right_panel(Some(kind));
                            }
                        }
                    });
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    close |= icons::button(ui, "x", TOUCH, false, tl!("Close panel")).clicked();
                });
            });
        }
        match drawer {
            Drawer::Tools => {
                app.left_open = true;
                crate::panels::left_panel(app, &mut ui);
                // The panel's own close button closes the drawer, not the desktop panel.
                if !app.left_open {
                    app.left_open = true;
                    close = true;
                }
            }
            Drawer::Panels => {
                if app.right.is_none() {
                    app.choose_right_panel(Some(RightPanel::Pages));
                }
                crate::panels::right_panel(app, &mut ui);
                if app.right.is_none() {
                    close = true;
                }
            }
        }
        close |= scrim.clicked() && scrim.interact_pointer_pos().is_some_and(|p| !panel.contains(p));
    });
    if close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.compact_drawer = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_follows_width() {
        assert!(is_compact(390.0));
        assert!(is_compact(699.0));
        assert!(!is_compact(700.0));
        assert!(!is_compact(1280.0));
        for w in [0.0, -5.0, f32::NAN, f32::INFINITY] {
            assert!(!is_compact(w), "{w}");
        }
    }

    #[test]
    fn drawer_leaves_a_strip_to_tap() {
        assert_eq!(drawer_width(390.0), 342.0);
        assert_eq!(drawer_width(690.0), DRAWER_MAX);
        assert_eq!(drawer_width(10.0), 0.0);
        assert_eq!(drawer_width(f32::NAN), 0.0);
    }
}
