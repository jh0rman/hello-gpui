mod headers_editor;
mod response_panel;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use gpui::{ClickEvent, Context, Entity, SharedString, Window, div, prelude::*, px, rgb};
use gpui_component::{
    Selectable,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};

use crate::network_module::{self, HttpRequest};
use crate::storage_module::{self, CollectionNode, SavedRequest};
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

// ── Sidebar tree helpers ───────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum SidebarKind {
    Folder { expanded: bool },
    Request,
}

#[derive(Clone)]
struct SidebarItem {
    name: String,
    path: PathBuf,
    kind: SidebarKind,
    depth: usize,
}

/// Flattens the visible portion of the collection tree into a linear list.
fn flatten_visible(
    nodes: &[CollectionNode],
    depth: usize,
    expanded: &HashSet<PathBuf>,
    out: &mut Vec<SidebarItem>,
) {
    for node in nodes {
        match node {
            CollectionNode::Folder { name, path, children } => {
                let is_expanded = expanded.contains(path);
                out.push(SidebarItem {
                    name: name.clone(),
                    path: path.clone(),
                    kind: SidebarKind::Folder { expanded: is_expanded },
                    depth,
                });
                if is_expanded {
                    flatten_visible(children, depth + 1, expanded, out);
                }
            }
            CollectionNode::Request { name, path } => {
                out.push(SidebarItem {
                    name: name.clone(),
                    path: path.clone(),
                    kind: SidebarKind::Request,
                    depth,
                });
            }
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
    collection_dir: PathBuf,  // where Save writes new files
    tree: Vec<CollectionNode>,
    expanded: HashSet<PathBuf>,

    // Active environment — loaded from env.json in the current collection folder.
    active_env: HashMap<String, String>,
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
        let tree = storage_module::load_collection_tree(&storage_module::makako_root_dir());
        let active_env = storage_module::load_env(&collection_dir);

        Self {
            method: HttpMethod::Get,
            url_input,
            headers_editor,
            body_input,
            response_panel,
            collection_dir,
            tree,
            expanded: HashSet::new(),
            active_env,
        }
    }

    fn refresh_tree(&mut self) {
        self.tree = storage_module::load_collection_tree(&storage_module::makako_root_dir());
    }

    fn visible_sidebar_items(&self) -> Vec<SidebarItem> {
        let mut items = vec![];
        flatten_visible(&self.tree, 0, &self.expanded, &mut items);
        items
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
            let env = &this.active_env;

            let url = storage_module::interpolate(
                &this.url_input.read(cx).value(),
                env,
            );
            let method = this.method.label().to_string();
            let headers = this.headers_editor
                .read(cx)
                .headers(cx)
                .into_iter()
                .map(|(k, v)| (k, storage_module::interpolate(&v, env)))
                .collect();
            let body = {
                let raw = this.body_input.read(cx).value().to_string();
                let b = storage_module::interpolate(&raw, env);
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
            let name = url
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or("Untitled")
                .to_string();

            let req = SavedRequest {
                name,
                method: this.method.label().to_string(),
                url,
                headers: this.headers_editor.read(cx).headers(cx),
                body: this.body_input.read(cx).value().to_string(),
            };

            let dir = this.collection_dir.clone();
            match storage_module::save_request(&dir, &req) {
                Ok(_) => this.refresh_tree(),
                Err(e) => eprintln!("[Makako] save error: {e}"),
            }
            cx.notify();
        });

        // ── Sidebar tree: flatten visible nodes then create one listener each ─
        let items = self.visible_sidebar_items();

        let sidebar_listeners: Vec<_> = items
            .iter()
            .map(|item| {
                let path = item.path.clone();
                let is_folder = matches!(item.kind, SidebarKind::Folder { .. });
                cx.listener(move |this, _: &ClickEvent, window, cx| {
                    if is_folder {
                        if this.expanded.contains(&path) {
                            this.expanded.remove(&path);
                        } else {
                            this.expanded.insert(path.clone());
                        }
                        cx.notify();
                    } else {
                        let Ok(req) = storage_module::load_request(&path) else {
                            return;
                        };
                        // Load the env.json from this request's parent folder.
                        if let Some(parent) = path.parent() {
                            this.active_env = storage_module::load_env(parent);
                        }
                        this.method = HttpMethod::from_str(&req.method);
                        this.url_input.update(cx, |s, cx| s.set_value(req.url, window, cx));
                        this.body_input.update(cx, |s, cx| s.set_value(req.body, window, cx));
                        this.headers_editor
                            .update(cx, |he, cx| he.load_headers(req.headers, window, cx));
                        cx.notify();
                    }
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
                    .pt_3()
                    // Section label
                    .child(
                        div()
                            .px_3()
                            .pb_2()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(0x555577))
                            .child("COLLECTIONS"),
                    )
                    // Tree rows
                    .children(
                        items
                            .iter()
                            .zip(sidebar_listeners)
                            .enumerate()
                            .map(|(i, (item, on_click))| {
                                let indent = px(8.0 + item.depth as f32 * 16.0);
                                let (icon, icon_color, name_color) = match &item.kind {
                                    SidebarKind::Folder { expanded: true } => {
                                        ("▾", rgb(0x7788bb), rgb(0xccccee))
                                    }
                                    SidebarKind::Folder { expanded: false } => {
                                        ("▸", rgb(0x556699), rgb(0xaaaacc))
                                    }
                                    SidebarKind::Request => ("·", rgb(0x445577), rgb(0x9999bb)),
                                };

                                div()
                                    .id(("tree-item", i))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1()
                                    .pl(indent)
                                    .pr_2()
                                    .py_1()
                                    .mx_1()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x252540)))
                                    .on_click(on_click)
                                    .child(
                                        div()
                                            .w(px(12.0))
                                            .text_xs()
                                            .text_color(icon_color)
                                            .child(icon),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_sm()
                                            .text_color(name_color)
                                            .child(SharedString::from(item.name.clone())),
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
