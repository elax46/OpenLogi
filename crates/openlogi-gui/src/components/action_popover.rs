//! Button-to-action picker.
//!
//! For each [`ButtonId`] we render a small card showing the current binding;
//! clicking it opens a popover with the [`Action`] catalog. Picking an action
//! writes it back to [`AppState::button_bindings`] and dismisses.
//!
//! Per UI.md Phase 4. The Phase 6 mouse model will eventually replace these
//! cards with real hotspots; the binding/popover plumbing stays.

use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, BorrowAppContext as _, Context, Entity, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, StatefulInteractiveElement as _, Styled, Window, div, px,
    rgb,
};
use gpui_component::{
    button::Button,
    h_flex,
    popover::{Popover, PopoverState},
    v_flex,
};

use crate::data::mouse_buttons::{Action, ButtonId};
use crate::state::AppState;
use crate::theme::{ACCENT_BLUE, SURFACE, SURFACE_HOVER, TEXT_MUTED, TEXT_PRIMARY};

const POPOVER_W: f32 = 200.;

pub struct ActionPopoverRow {
    buttons: Vec<ButtonId>,
}

impl ActionPopoverRow {
    pub fn new(buttons: Vec<ButtonId>) -> Self {
        Self { buttons }
    }

    pub fn default_row() -> Self {
        Self::new(vec![
            ButtonId::LeftClick,
            ButtonId::RightClick,
            ButtonId::Back,
            ButtonId::Forward,
        ])
    }
}

impl Render for ActionPopoverRow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bindings = cx
            .try_global::<AppState>()
            .map(|s| s.button_bindings.clone())
            .unwrap_or_default();
        let self_entity = cx.entity();

        h_flex()
            .gap_4()
            .items_center()
            .children(self.buttons.iter().enumerate().map(|(idx, &btn)| {
                let current = bindings
                    .get(&btn)
                    .map_or("Unbound", |a| a.label())
                    .to_string();
                let row = self_entity.clone();
                Popover::new(("action-popover", idx))
                    .anchor(Anchor::TopLeft)
                    .mouse_button(MouseButton::Left)
                    .trigger(card_trigger(idx, btn, &current))
                    .content(move |state, window, cx| action_list(btn, &row, state, window, cx))
                    .into_any_element()
            }))
    }
}

/// The visible card the user clicks. Uses Button so it picks up the
/// Popover's `Selectable` styling when the popover is open.
fn card_trigger(idx: usize, btn: ButtonId, current: &str) -> Button {
    let label = format!("{}\n→ {current}", btn.label());
    Button::new(("action-trigger", idx))
        .label(label)
        .outline()
        .compact()
}

fn action_list(
    btn: ButtonId,
    row: &Entity<ActionPopoverRow>,
    _state: &mut PopoverState,
    _window: &mut Window,
    cx: &mut Context<PopoverState>,
) -> AnyElement {
    let popover = cx.entity().downgrade();
    let current = cx
        .try_global::<AppState>()
        .and_then(|s| s.button_bindings.get(&btn).cloned());

    let items: Vec<AnyElement> = Action::catalog()
        .into_iter()
        .enumerate()
        .map(|(item_idx, action)| {
            let is_selected = current.as_ref() == Some(&action);
            let label = action.label().to_string();
            let row = row.clone();
            let popover = popover.clone();
            let action = Rc::new(action);
            div()
                .id(("action-item", item_idx))
                .w_full()
                .px_3()
                .py_1p5()
                .rounded_md()
                .text_sm()
                .text_color(rgb(if is_selected {
                    ACCENT_BLUE
                } else {
                    TEXT_PRIMARY
                }))
                .bg(rgb(if is_selected { SURFACE_HOVER } else { SURFACE }))
                .hover(|s| s.bg(rgb(SURFACE_HOVER)))
                .child(label)
                .on_click(move |_event, window, cx| {
                    let action = (*action).clone();
                    cx.update_global::<AppState, _>(|state, _| {
                        state.button_bindings.insert(btn, action);
                    });
                    row.update(cx, |_, cx| cx.notify());
                    if let Some(p) = popover.upgrade() {
                        p.update(cx, |s, cx| s.dismiss(window, cx));
                    }
                })
                .into_any_element()
        })
        .collect();

    v_flex()
        .min_w(px(POPOVER_W))
        .gap_1()
        .p_2()
        .child(
            div()
                .text_xs()
                .text_color(rgb(TEXT_MUTED))
                .px_2()
                .pb_1()
                .child(format!("Bind {}", btn.label())),
        )
        .children(items)
        .into_any_element()
}
