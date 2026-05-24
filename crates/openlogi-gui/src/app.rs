//! Root view: header (device carousel), body (mouse model area and
//! configuration panel), footer (settings / version).
//!
//! Body currently hosts the Phase 2 [`DpiPanel`] beside the Phase 4
//! [`ActionPopoverRow`]; the surrounding layout (mouse model + multi-tab
//! config) is being filled in across the remaining UI.md phases.

use gpui::{
    AppContext as _, Context, Entity, FontWeight, IntoElement, ParentElement, Render, Styled,
    Window, div, px, rgb,
};
use gpui_component::{ActiveTheme, h_flex, v_flex};
use openlogi_core::device::DeviceInventory;

use crate::components::action_popover::ActionPopoverRow;
use crate::components::device_carousel::DeviceCarousel;
use crate::components::dpi_panel::DpiPanel;
use crate::state::AppState;
use crate::theme::{BG_DARK, BORDER, FOOTER_H, HEADER_H, TEXT_MUTED, TEXT_PRIMARY};

pub struct AppView {
    carousel: Entity<DeviceCarousel>,
    dpi_panel: Entity<DpiPanel>,
    action_row: Entity<ActionPopoverRow>,
}

impl AppView {
    pub fn new(inventories: &[DeviceInventory], cx: &mut Context<Self>) -> Self {
        if !cx.has_global::<AppState>() {
            cx.set_global(AppState::new());
        }
        let carousel = cx.new(|cx| DeviceCarousel::new(inventories, cx));
        let dpi_panel = cx.new(DpiPanel::new);
        let action_row = cx.new(|_| ActionPopoverRow::default_row());
        Self {
            carousel,
            dpi_panel,
            action_row,
        }
    }
}

impl Render for AppView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(rgb(BG_DARK))
            .text_color(rgb(TEXT_PRIMARY))
            .child(header(&self.carousel))
            .child(body(&self.dpi_panel, &self.action_row))
            .child(footer(cx))
    }
}

fn header(carousel: &Entity<DeviceCarousel>) -> impl IntoElement {
    h_flex()
        .h(px(HEADER_H))
        .w_full()
        .px_5()
        .gap_4()
        .items_center()
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .child("OpenLogi"),
        )
        .child(div().flex_1().min_w_0().child(carousel.clone()))
}

fn body(dpi_panel: &Entity<DpiPanel>, action_row: &Entity<ActionPopoverRow>) -> impl IntoElement {
    h_flex()
        .flex_1()
        .w_full()
        .min_h_0()
        .items_start()
        .justify_center()
        .gap_10()
        .p_8()
        .child(
            v_flex()
                .gap_4()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT_MUTED))
                        .child("Button bindings"),
                )
                .child(action_row.clone()),
        )
        .child(dpi_panel.clone())
}

fn footer(cx: &Context<AppView>) -> impl IntoElement {
    let theme = cx.theme();
    h_flex()
        .h(px(FOOTER_H))
        .w_full()
        .px_5()
        .gap_4()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("Settings · About"),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(concat!("v", env!("CARGO_PKG_VERSION"))),
        )
}
