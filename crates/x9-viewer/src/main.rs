//! X9 Viewer — a fast, native viewer/editor for ANSI X9.37 / X9.100-187 files.

mod app;
mod image_view;

use gpui_kit::{
    self as gpui, px, size, App, AppContext as _, Bounds, WindowBounds, WindowOptions,
};

fn main() {
    let path = std::env::args().nth(1).map(std::path::PathBuf::from);
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(move |cx: &mut App| {
            gpui::init(cx);
            cx.activate(true);
            let bounds = Bounds::centered(None, size(px(1440.), px(900.)), cx);
            gpui::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                cx,
                |window, cx| cx.new(|cx| app::X9App::new(path.clone(), window, cx)),
            )
            .expect("open window");
        });
}
