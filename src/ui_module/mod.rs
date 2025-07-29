mod headers_editor;
mod response_panel;

use std::path::PathBuf;

use gpui::{ClickEvent, Context, Entity, SharedString, Window, div, prelude::*, px, rgb};
use gpui_component::{
    Selectable,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};

use crate::network_module::{self, HttpRequest};
use crate::storage_module::{self, SavedRequest};
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

    fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            _ => HttpMethod::Get,
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

    // Sidebar state
    collection_dir: PathBuf,
    saved_requests: Vec<(String, PathBuf)>, // (display name, file path)
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
        let collection_dir = storage_module::default_collection_dir();
        let saved_requests = storage_module::list_requests(&collection_dir);

        Self {
            method: HttpMethod::Get,
            url_input,
            headers_editor,
            body_input,
            response_panel,
            collection_dir,
            saved_requests,
        }
    }

    /// Reloads the sidebar list from disk.
    fn refresh_sidebar(&mut self) {
        self.saved_requests = storage_module::list_requests(&self.collection_dir);
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let method = self.method;

        // ── Method listeners ─────────────────────────────────────────────────
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

        // ── Send ─────────────────────────────────────────────────────────────
        let on_send = cx.listener(|this, _: &ClickEvent, _, cx| {
            let url = this.url_input.read(cx).value().to_string();
            let method = this.method.label().to_string();
            let headers = this.headers_editor.read(cx).headers(cx);
            let body = {
                let b = this.body_input.read(cx).value().to_string();
                if b.trim().is_empty() { None } else { Some(b) }
            };

            let req = HttpRequest { method, url, headers, body };

            this.response_panel.update(cx, |panel, cx| {
                panel.loading = true;
                panel.response = None;
                panel.error = None;
                cx.notify();
            });

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

        // ── Save ─────────────────────────────────────────────────────────────
        let on_save = cx.listener(|this, _: &ClickEvent, _, cx| {
            let url = this.url_input.read(cx).value().to_string();
            let name = if url.is_empty() {
                "Untitled".to_string()
            } else {
                // Use last path segment of URL as default name.
                url.trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap_or("Untitled")
                    .to_string()
            };

            let req = SavedRequest {
                name,
                method: this.method.label().to_string(),
                url,
                headers: this.headers_editor.read(cx).headers(cx),
                body: this.body_input.read(cx).value().to_string(),
            };

            let dir = this.collection_dir.clone();
            match storage_module::save_request(&dir, &req) {
                Ok(_) => this.refresh_sidebar(),
                Err(e) => eprintln!("[Makako] save error: {e}"),
            }
            cx.notify();
        });

        // ── Sidebar load listeners (one per saved request) ───────────────────
        let load_listeners: Vec<_> = self
            .saved_requests
            .iter()
            .map(|(_, path)| {
                let path = path.clone();
                cx.listener(move |this, _: &ClickEvent, window, cx| {
                    let Ok(req) = storage_module::load_request(&path) else {
                        return;
                    };
                    this.method = HttpMethod::from_str(&req.method);
                    this.url_input.update(cx, |s, cx| s.set_value(req.url, window, cx));
                    this.body_input.update(cx, |s, cx| s.set_value(req.body, window, cx));
                    this.headers_editor
                        .update(cx, |he, cx| he.load_headers(req.headers, window, cx));
                    cx.notify();
                })
            })
            .collect();

        // ── Layout ───────────────────────────────────────────────────────────
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
                    .flex()
                    .flex_col()
                    .bg(rgb(0x1a1a2e))
                    .p_3()
                    // Section label
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(0x555577))
                            .pb_2()
                            .child("COLLECTIONS"),
                    )
                    // Request list
                    .children(
                        self.saved_requests
                            .iter()
                            .zip(load_listeners)
                            .enumerate()
                            .map(|(i, ((name, _), on_load))| {
                                div()
                                    .id(("sidebar-item", i))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x2a2a44)))
                                    .on_click(on_load)
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x6677aa))
                                            .child(
                                                self.saved_requests[i]
                                                    .0
                                                    .chars()
                                                    .take(3)
                                                    .collect::<String>()
                                                    .to_uppercase(),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_sm()
                                            .text_color(rgb(0xaaaacc))
                                            .child(SharedString::from(name.clone())),
                                    )
                            }),
                    ),
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
                                Button::new("btn-save")
                                    .label("Save")
                                    .ghost()
                                    .on_click(on_save),
                            )
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
