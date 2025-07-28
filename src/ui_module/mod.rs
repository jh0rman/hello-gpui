mod headers_editor;
mod response_panel;

use gpui::{ClickEvent, Context, Entity, Window, div, prelude::*, px, rgb};
use gpui_component::{
    Selectable,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};

use crate::network_module::{self, HttpRequest};
use headers_editor::HeadersEditor;
use response_panel::ResponsePanel;

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
    body_input: Entity<InputState>,
    response_panel: Entity<ResponsePanel>,
}

impl AppView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let url_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("https://api.example.com/resource")
        });
        let headers_editor = cx.new(|cx| HeadersEditor::new(window, cx));
        let body_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("json")
                .placeholder("// JSON request body")
        });
        let response_panel = cx.new(|_cx| ResponsePanel::new());
        Self {
            method: HttpMethod::Get,
            url_input,
            headers_editor,
            body_input,
            response_panel,
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
            let url = this.url_input.read(cx).value().to_string();
            let method = this.method.label().to_string();
            let headers = this.headers_editor.read(cx).headers(cx);
            let body = {
                let b = this.body_input.read(cx).value().to_string();
                if b.trim().is_empty() { None } else { Some(b) }
            };

            let req = HttpRequest { method, url, headers, body };

            // Mark the panel as loading.
            this.response_panel.update(cx, |panel, cx| {
                panel.loading = true;
                panel.response = None;
                panel.error = None;
                cx.notify();
            });

            // Spawn async task: bridge blocking reqwest via oneshot channel.
            cx.spawn(async move |view, async_cx| {
                let (tx, rx) = futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(network_module::execute(req));
                });

                let result = rx.await.unwrap_or_else(|_| Err("thread panicked".to_string()));

                view.update(async_cx, |this, cx| {
                    this.response_panel.update(cx, |panel, cx| {
                        panel.loading = false;
                        match result {
                            Ok(resp) => {
                                panel.response = Some(resp);
                                panel.error = None;
                            }
                            Err(e) => {
                                panel.error = Some(e);
                                panel.response = None;
                            }
                        }
                        cx.notify();
                    });
                })
                .ok();
            })
            .detach();
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
                    // Body section
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .text_sm()
                                    .text_color(rgb(0x7777aa))
                                    .child("Body"),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .px_3()
                                    .pb_3()
                                    .child(Input::new(&self.body_input).h_full()),
                            ),
                    ),
            )
            // ── Response panel ────────────────────────────────────
            .child(
                div()
                    .w(px(420.0))
                    .h_full()
                    .child(self.response_panel.clone()),
            )
    }
}
