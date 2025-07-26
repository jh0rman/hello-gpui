use gpui::{App, ClickEvent, Context, Entity, Window, div, prelude::*, px, rgb};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};

// ── HeaderRow ─────────────────────────────────────────────────────────────────

struct HeaderRow {
    key: Entity<InputState>,
    value: Entity<InputState>,
}

impl HeaderRow {
    fn new(window: &mut Window, cx: &mut Context<HeadersEditor>) -> Self {
        let key = cx.new(|cx| InputState::new(window, cx).placeholder("Header name"));
        let value = cx.new(|cx| InputState::new(window, cx).placeholder("Value"));
        Self { key, value }
    }
}

// ── HeadersEditor ─────────────────────────────────────────────────────────────

pub struct HeadersEditor {
    rows: Vec<HeaderRow>,
}

impl HeadersEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            rows: vec![HeaderRow::new(window, cx)],
        }
    }

    /// Returns non-empty (key, value) pairs for use in HTTP requests.
    pub fn headers(&self, cx: &App) -> Vec<(String, String)> {
        self.rows
            .iter()
            .map(|r| {
                (
                    r.key.read(cx).value().to_string(),
                    r.value.read(cx).value().to_string(),
                )
            })
            .filter(|(k, _)| !k.trim().is_empty())
            .collect()
    }
}

impl Render for HeadersEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // ── Listeners ── must be created before the builder chain ──────────
        let on_add = cx.listener(|this, _: &ClickEvent, window, cx| {
            this.rows.push(HeaderRow::new(window, cx));
            cx.notify();
        });

        // One remove-listener per existing row (captures index by copy).
        let remove_listeners: Vec<_> = (0..self.rows.len())
            .map(|i| {
                cx.listener(move |this, _: &ClickEvent, _, cx| {
                    this.rows.remove(i);
                    cx.notify();
                })
            })
            .collect();

        // ── Layout ─────────────────────────────────────────────────────────
        div()
            .flex()
            .flex_col()
            .gap_1()
            // Column labels
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .text_color(rgb(0x666688))
                    .text_sm()
                    .child(div().flex_1().child("Key"))
                    .child(div().flex_1().child("Value"))
                    .child(div().w(px(28.0))), // spacer aligns with delete btn
            )
            // Rows
            .children(
                self.rows
                    .iter()
                    .zip(remove_listeners)
                    .enumerate()
                    .map(|(i, (row, on_remove))| {
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .child(div().flex_1().child(Input::new(&row.key)))
                            .child(div().flex_1().child(Input::new(&row.value)))
                            .child(
                                Button::new(("header-del", i))
                                    .label("×")
                                    .ghost()
                                    .on_click(on_remove),
                            )
                    }),
            )
            // Add row button
            .child(
                div().px_3().child(
                    Button::new("btn-add-header")
                        .label("+ Add Header")
                        .ghost()
                        .on_click(on_add),
                ),
            )
    }
}
