mod headers_editor;

use gpui::{ClickEvent, Context, Entity, Window, div, prelude::*, px, rgb};
use gpui_component::{
    Selectable,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};

use headers_editor::HeadersEditor;

// ── HTTP method ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl HttpMethod {
    fn label(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
        }
    }
}

// ── AppView ───────────────────────────────────────────────────────────────────

pub struct AppView {
    method: HttpMethod,
    url_input: Entity<InputState>,
    headers_editor: Entity<HeadersEditor>,
}

impl AppView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let url_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("https://api.example.com/resource")
        });
        let headers_editor = cx.new(|cx| HeadersEditor::new(window, cx));
        Self {
            method: HttpMethod::Get,
            url_input,
            headers_editor,
        }
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Pre-create all click listeners before the builder chain.
        let method = self.method;

        let on_get = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.method = HttpMethod::Get;
            cx.notify();
        });
        let on_post = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.method = HttpMethod::Post;
            cx.notify();
        });
        let on_put = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.method = HttpMethod::Put;
            cx.notify();
        });
        let on_delete = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.method = HttpMethod::Delete;
            cx.notify();
        });
        let on_send = cx.listener(|this, _: &ClickEvent, _, cx| {
            let url = this.url_input.read(cx).value();
            let headers = this.headers_editor.read(cx).headers(cx);
            println!("[Makako] {} {}", this.method.label(), url);
            for (k, v) in &headers {
                println!("  {}: {}", k, v);
            }
        });

        div()
            .flex()
            .flex_row()
            .w_full()
            .h_full()
            // ── Sidebar ───────────────────────────────────────────
            .child(
                div()
                    .w(px(240.0))
                    .h_full()
                    .bg(rgb(0x1a1a2e))
                    .p_4()
                    .text_color(rgb(0x8888aa))
                    .child("Colecciones"),
            )
            // ── Central editor ────────────────────────────────────
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .bg(rgb(0x24243e))
                    // Request bar
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .p_3()
                            .bg(rgb(0x1e1e32))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .child(
                                        Button::new("btn-get")
                                            .label("GET")
                                            .ghost()
                                            .selected(method == HttpMethod::Get)
                                            .on_click(on_get),
                                    )
                                    .child(
                                        Button::new("btn-post")
                                            .label("POST")
                                            .ghost()
                                            .selected(method == HttpMethod::Post)
                                            .on_click(on_post),
                                    )
                                    .child(
                                        Button::new("btn-put")
                                            .label("PUT")
                                            .ghost()
                                            .selected(method == HttpMethod::Put)
                                            .on_click(on_put),
                                    )
                                    .child(
                                        Button::new("btn-delete")
                                            .label("DELETE")
                                            .ghost()
                                            .selected(method == HttpMethod::Delete)
                                            .on_click(on_delete),
                                    ),
                            )
                            .child(div().flex_1().child(Input::new(&self.url_input)))
                            .child(
                                Button::new("btn-send")
                                    .label("Send")
                                    .primary()
                                    .on_click(on_send),
                            ),
                    )
                    // Headers section
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .border_b_1()
                            .border_color(rgb(0x2e2e4a))
                            // Section label
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .text_sm()
                                    .text_color(rgb(0x7777aa))
                                    .child("Headers"),
                            )
                            .child(self.headers_editor.clone()),
                    )
                    // Body placeholder (next iteration)
                    .child(
                        div()
                            .flex_1()
                            .px_3()
                            .py_2()
                            .text_sm()
                            .text_color(rgb(0x444466))
                            .child("Body"),
                    ),
            )
            // ── Response panel ────────────────────────────────────
            .child(
                div()
                    .w(px(420.0))
                    .h_full()
                    .bg(rgb(0x0f3460))
                    .p_4()
                    .text_color(rgb(0x88bbdd))
                    .child("Respuesta"),
            )
    }
}
