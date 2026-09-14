//! Category strip config panel — floating panel for configuring which buttons
//! sit above the list and in what order.
//!
//! --- Right-click the filter bar to open. ---
//! --- Each row: circular checkbox (show/hide) + icon + label + up/down arrows. ---
//! --- Two sections, because the strip mixes two things that are configured ---
//! --- differently: the built-in content types are a fixed set that can only ---
//! --- be shown or hidden, while tags are added to the strip by pinning them. ---

use gpui::prelude::*;
use gpui::*;
use gpui_component::scroll::ScrollableElement;

use crate::core::i18n_keys::I18nKey;
use crate::state::app::AppState;

use super::filter_bar::{filter_type_display, FilterBar};
use super::sidebar::parse_tag_color;
use super::theme::ClippiTheme;

/// One reorder arrow. Disabled at the ends of its section — the strip is short
/// and the list has a top and a bottom, so there is nothing to wrap to.
fn arrow_button<F>(
    glyph: &'static str,
    disabled: bool,
    bg: Rgba,
    hover_bg: Rgba,
    fg: Rgba,
    on_click: F,
) -> Div
where
    F: Fn(&mut App) + 'static,
{
    div()
        .w(px(22.))
        .h(px(22.))
        .rounded(px(4.))
        .flex()
        .items_center()
        .justify_center()
        .bg(bg)
        .cursor(CursorStyle::PointingHand)
        .when(disabled, |el| el.opacity(0.2))
        .when(!disabled, |el| {
            el.hover(|el| el.bg(hover_bg))
                .on_mouse_down(MouseButton::Left, move |_ev, _window, cx| {
                    cx.stop_propagation();
                    on_click(cx);
                })
        })
        .child(
            div()
                .text_size(px(12.))
                .font_family("iconfont")
                .text_color(fg)
                .child(glyph),
        )
}

/// Section heading inside the panel.
fn section_label(text: &'static str, fg: Rgba) -> Div {
    div()
        .pt(px(4.))
        .pb(px(2.))
        .px(px(4.))
        .text_size(px(10.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(fg)
        .child(text)
}

pub struct TypeFilterConfigPanel {
    state: Entity<AppState>,
    filter_bar: Entity<FilterBar>,
}

impl TypeFilterConfigPanel {
    pub fn new(
        state: Entity<AppState>,
        filter_bar: Entity<FilterBar>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self { state, filter_bar }
    }

    pub fn close(&self, cx: &mut App) {
        self.filter_bar
            .update(cx, |bar, cx| bar.close_filter_config(cx));
    }

    fn toggle_visible(&self, key: &str, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            if let Some(entry) = s
                .settings
                .type_filter_config
                .iter_mut()
                .find(|e| e.key == key)
            {
                let was_visible = entry.visible;
                entry.visible = !entry.visible;
                // If hiding a currently active filter, deactivate it
                if was_visible && !entry.visible && s.filters.is_type_active(key) {
                    s.filters.toggle_type(key);
                }
                s.settings.save();
                s.reload_items();
            }
        });
        self.filter_bar.update(cx, |_b, cx| cx.notify());
    }

    fn move_up(&self, index: usize, cx: &mut App) {
        if index == 0 {
            return;
        }
        self.state.update(cx, |s, _cx| {
            s.settings.type_filter_config.swap(index, index - 1);
            s.settings.save();
            s.reload_items();
        });
        self.filter_bar.update(cx, |_b, cx| cx.notify());
    }

    /// Pin or unpin a tag — this is what adds a tag button to the strip or
    /// takes it away. Deliberately the same `toggle_pinned_tag` the sidebar
    /// calls: one action, one behaviour, whichever way you reach it.
    fn toggle_pinned(&self, tag_id: i64, cx: &mut Context<Self>) {
        self.state.update(cx, |s, _cx| {
            s.toggle_pinned_tag(tag_id);
        });
        self.filter_bar.update(cx, |_b, cx| cx.notify());
        cx.notify();
    }

    fn move_pinned(&self, tag_id: i64, delta: isize, cx: &mut Context<Self>) {
        self.state.update(cx, |s, _cx| {
            s.move_pinned_tag(tag_id, delta);
        });
        self.filter_bar.update(cx, |_b, cx| cx.notify());
        cx.notify();
    }

    fn move_down(&self, index: usize, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            let len = s.settings.type_filter_config.len();
            if index + 1 >= len {
                return;
            }
            s.settings.type_filter_config.swap(index, index + 1);
            s.settings.save();
            s.reload_items();
        });
        self.filter_bar.update(cx, |_b, cx| cx.notify());
    }
}

impl Render for TypeFilterConfigPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.state.read(cx);
        let theme = ClippiTheme::from_setting(&app_state.settings.theme, Some(window.appearance()));
        let config = app_state.settings.type_filter_config.clone();
        let pinned_ids = app_state.settings.pinned_tag_ids.clone();
        let tags = app_state.tags.clone();
        let _ = app_state;

        // Pinned tags first, in the order they sit in the strip, then the rest —
        // the top of the section mirrors what the strip actually shows.
        let mut tag_rows: Vec<_> = pinned_ids
            .iter()
            .filter_map(|id| tags.iter().find(|t| t.id == *id).cloned())
            .collect();
        let pinned_count = tag_rows.len();
        tag_rows.extend(
            tags.iter()
                .filter(|t| !pinned_ids.contains(&t.id))
                .cloned(),
        );

        let accent = theme.accent;
        let text_1 = theme.text_1;
        let text_2 = theme.text_2;
        let text_3 = theme.text_3;
        let surface = theme.panel_surface;
        let sep_line = theme.panel_sep_line;
        let btn_hover = theme.btn_hover;
        let panel_border = if theme.bg == rgb(0x191a1b) {
            rgba(0xffffff14)
        } else {
            rgba(0x00000012)
        };
        let is_dark = theme.bg == rgb(0x191a1b);
        let arrow_bg = if is_dark {
            rgba(0xffffff0a)
        } else {
            rgba(0x00000008)
        };
        let arrow_hover = if is_dark {
            rgba(0xffffff18)
        } else {
            rgba(0x00000014)
        };

        let this_entity = cx.entity().clone();
        let len = config.len();

        div()
            .flex()
            .flex_col()
            .w(px(240.))
            .bg(surface)
            .border_color(panel_border)
            .border(px(1.))
            .rounded(px(8.))
            .shadow_lg()
            .p(px(8.))
            .gap(px(4.))
            // --- Title row ---
            .child({
                let this = this_entity.clone();
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(px(24.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(text_1)
                            .child(I18nKey::FilterConfigTitle.text()),
                    )
                    .child(
                        div()
                            .w(px(22.))
                            .h(px(22.))
                            .rounded(px(4.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor(CursorStyle::PointingHand)
                            .hover(|el| el.bg(btn_hover))
                            .on_mouse_down(MouseButton::Left, move |_ev, _window, cx| {
                                cx.stop_propagation();
                                this.update(cx, |panel, cx| panel.close(cx));
                            })
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .font_family("iconfont")
                                    .text_color(text_2)
                                    .child("\u{e7b7}"),
                            ),
                    )
            })
            // --- Separator ---
            .child(div().w_full().h(px(1.)).bg(sep_line))
            // --- Types section ---
            .child(section_label(I18nKey::FilterConfigTypes.text(), text_3))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .children(config.iter().enumerate().map(|(i, entry)| {
                        let key = entry.key.clone();
                        let visible = entry.visible;
                        let is_first = i == 0;
                        let is_last = i + 1 >= len;

                        // Look up display info from FILTER_TYPES
                        let (icon, label) =
                            filter_type_display(&key).unwrap_or(("\u{e606}", key.clone()));

                        let key_for_toggle = key.clone();

                        // Radio-style icon: \u{e831} = empty circle, \u{e61f} = filled circle
                        let checkbox = div()
                            .text_size(px(12.))
                            .font_family("iconfont")
                            .text_color(if visible { accent } else { text_3 })
                            .flex_shrink_0()
                            .child(if visible { "\u{e61f}" } else { "\u{e831}" });

                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .h(px(28.))
                            .px(px(4.))
                            .rounded(px(4.))
                            .cursor(CursorStyle::PointingHand)
                            .hover(|el| el.bg(btn_hover))
                            .on_mouse_down(MouseButton::Left, {
                                let this = this_entity.clone();
                                let k = key_for_toggle.clone();
                                move |_ev, _window, cx| {
                                    cx.stop_propagation();
                                    this.update(cx, |panel, cx| panel.toggle_visible(&k, cx));
                                }
                            })
                            // Checkbox
                            .child(checkbox)
                            // Icon
                            .child(
                                div()
                                    .ml(px(6.))
                                    .text_size(px(12.))
                                    .font_family("iconfont")
                                    .text_color(if visible { text_2 } else { text_3 })
                                    .child(icon.to_string()),
                            )
                            // Label
                            .child(
                                div()
                                    .ml(px(4.))
                                    .text_size(px(11.))
                                    .flex_1()
                                    .text_color(if visible { text_1 } else { text_3 })
                                    .child(label),
                            )
                            // Up arrow
                            .child(arrow_button(
                                "\u{e665}",
                                is_first,
                                arrow_bg,
                                arrow_hover,
                                text_2,
                                {
                                    let this = this_entity.clone();
                                    move |cx: &mut App| {
                                        this.update(cx, |panel, cx| panel.move_up(i, cx));
                                    }
                                },
                            ))
                            // Down arrow
                            .child(div().ml(px(2.)).child(arrow_button(
                                "\u{e666}",
                                is_last,
                                arrow_bg,
                                arrow_hover,
                                text_2,
                                {
                                    let this = this_entity.clone();
                                    move |cx: &mut App| {
                                        this.update(cx, |panel, cx| panel.move_down(i, cx));
                                    }
                                },
                            )))
                    })),
            )
            // --- Tags section ---
            .child(section_label(I18nKey::FilterConfigTags.text(), text_3))
            .when(tag_rows.is_empty(), |el| {
                el.child(
                    div()
                        .px(px(4.))
                        .py(px(6.))
                        .text_size(px(11.))
                        .text_color(text_3)
                        .child(I18nKey::FilterConfigNoTags.text()),
                )
            })
            .when(!tag_rows.is_empty(), |el| {
                el.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        // --- A long tag list would grow the panel past the ---
                        // --- window; cap it and let it scroll instead. ---
                        .max_h(px(180.))
                        .overflow_y_scrollbar()
                        .children(tag_rows.iter().enumerate().map(|(i, tag)| {
                            let tag_id = tag.id;
                            let pinned = i < pinned_count;
                            let is_first = i == 0;
                            let is_last = i + 1 >= pinned_count;

                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .h(px(28.))
                                .px(px(4.))
                                .rounded(px(4.))
                                .cursor(CursorStyle::PointingHand)
                                .hover(|el| el.bg(btn_hover))
                                .on_mouse_down(MouseButton::Left, {
                                    let this = this_entity.clone();
                                    move |_ev, _window, cx| {
                                        cx.stop_propagation();
                                        this.update(cx, |panel, cx| panel.toggle_pinned(tag_id, cx));
                                    }
                                })
                                // Checkbox — pinned means "on the strip"
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_family("iconfont")
                                        .text_color(if pinned { accent } else { text_3 })
                                        .flex_shrink_0()
                                        .child(if pinned { "\u{e61f}" } else { "\u{e831}" }),
                                )
                                // The tag's own colour, the same dot the strip draws
                                .child(
                                    div()
                                        .ml(px(6.))
                                        .w(px(8.))
                                        .h(px(8.))
                                        .rounded_full()
                                        .flex_shrink_0()
                                        .bg(parse_tag_color(&tag.color))
                                        .when(!pinned, |el| el.opacity(0.4)),
                                )
                                .child(
                                    div()
                                        .ml(px(6.))
                                        .text_size(px(11.))
                                        .flex_1()
                                        .min_w(px(0.))
                                        .overflow_hidden()
                                        .text_color(if pinned { text_1 } else { text_3 })
                                        .child(tag.name.clone()),
                                )
                                // Reordering only means something for a tag that
                                // is on the strip; an unpinned one has no place
                                // in it to move.
                                .child(arrow_button(
                                    "\u{e665}",
                                    !pinned || is_first,
                                    arrow_bg,
                                    arrow_hover,
                                    text_2,
                                    {
                                        let this = this_entity.clone();
                                        move |cx: &mut App| {
                                            this.update(cx, |panel, cx| {
                                                panel.move_pinned(tag_id, -1, cx)
                                            });
                                        }
                                    },
                                ))
                                .child(div().ml(px(2.)).child(arrow_button(
                                    "\u{e666}",
                                    !pinned || is_last,
                                    arrow_bg,
                                    arrow_hover,
                                    text_2,
                                    {
                                        let this = this_entity.clone();
                                        move |cx: &mut App| {
                                            this.update(cx, |panel, cx| {
                                                panel.move_pinned(tag_id, 1, cx)
                                            });
                                        }
                                    },
                                )))
                        })),
                )
            })
    }
}
