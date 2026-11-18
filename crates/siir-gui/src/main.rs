//! GPUI bootstrap for the SIIR desktop front end.

mod ui;
mod view;

use std::path::PathBuf;

use gpui_kit::prelude::*;
use gpui_kit::{Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size};

use view::AppView;

fn main() {
    let dirs: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            ui::tokens::install(cx);
            let bounds = Bounds::centered(None, size(px(1100.0), px(720.0)), cx);
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(960.0), px(640.0))),
                    titlebar: Some(TitlebarOptions {
                        title: Some("SIIR".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                cx,
                move |window, cx| cx.new(|cx| AppView::new(dirs, window, cx)),
            )
            .expect("open the main window");
            cx.activate(true);
        });
}

#[cfg(test)]
mod tests {
    use gpui_kit::AssetSource;

    #[test]
    fn embeds_the_truck_icon() {
        let path = gpui_kit::assets::IconName::Truck.path();
        let asset = gpui_kit::assets::AllAssets.load(&path).unwrap();
        assert!(asset.is_some());
    }
}
