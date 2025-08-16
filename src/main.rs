mod network_module;
mod snippet_module;
mod storage_module;
mod ui_module;

use gpui::{px, size, App, AppContext, Application, Bounds, WindowBounds, WindowOptions};
use gpui_component::Root;
use ui_module::AppView;

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                // gpui_component requires Root as the window's top-level view.
                // Root sets up the theme, focus, dialogs and notification layers
                // used internally by Input, Button and other components.
                let app_view = cx.new(|cx| AppView::new(window, cx));
                cx.new(|cx| Root::new(app_view, window, cx))
            },
        )
        .unwrap();
    });
}
