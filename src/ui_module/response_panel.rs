use gpui::{Context, FontWeight, Window, div, prelude::*, rgb};

use crate::network_module::HttpResponse;

// ── ResponsePanel ──────────────────────────────────────────────────────────────

pub struct ResponsePanel {
    pub loading: bool,
    pub response: Option<HttpResponse>,
    pub error: Option<String>,
    pub snippet: Option<(String, String)>, // (lang_label, code)
}

impl ResponsePanel {
    pub fn new() -> Self {
        Self {
            loading: false,
            response: None,
            error: None,
            snippet: None,
        }
    }
}

impl Render for ResponsePanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p_4()
            .bg(rgb(0x16213e))
            // Panel header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .pb_3()
                    .border_b_1()
                    .border_color(rgb(0x2e2e4a))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x8888aa))
                            .child(if self.snippet.is_some() { "Code Snippet" } else { "Response" }),
                    ),
            )
            // Panel body — snippet / loading / error / response / empty
            .child(if let Some((lang, code)) = &self.snippet {
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .pt_3()
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x1a2a40))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x6699cc))
                            .child(lang.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .font_family("monospace")
                            .text_color(rgb(0xaaccee))
                            .child(code.clone()),
                    )
            } else if self.loading {
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0x6666aa))
                    .child("Sending…")
            } else if let Some(ref err) = self.error {
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .pt_3()
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x5a1a1a))
                            .text_color(rgb(0xff6b6b))
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .child("Error"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xcc4444))
                            .child(err.clone()),
                    )
            } else if let Some(ref resp) = self.response {
                let status = resp.status;
                let status_color = if status < 300 {
                    rgb(0x4ec994) // green
                } else if status < 500 {
                    rgb(0xf0a030) // orange
                } else {
                    rgb(0xff6b6b) // red
                };

                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .pt_3()
                    // Status + duration bar
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(0x1e3a2e))
                                    .text_color(status_color)
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .child(status.to_string()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x6666aa))
                                    .child(format!("{} ms", resp.duration_ms)),
                            ),
                    )
                    // Response body
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .font_family("monospace")
                            .text_color(rgb(0xaabbcc))
                            .child(resp.body.clone()),
                    )
            } else {
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0x444466))
                    .child("Send a request to see the response")
            })
    }
}
