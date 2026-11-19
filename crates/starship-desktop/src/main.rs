use std::{
    fs,
    path::{Path, PathBuf},
};

use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input};
use starship_core::Event;
use starship_render::View;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "build/previews".into()),
    );
    fs::create_dir_all(&output)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &font_paths(&fonts))?;
    let mut app = View::preview();
    let mut shot = |app: &View, name: &str, dark| {
        write_png(
            &output.join(name),
            WIDTH,
            HEIGHT,
            &frame(app, &mut canvas, dark),
        )
    };
    assert_eq!(app.input(Input::Tap(160.0, 1280.0)), Change::Partial);
    shot(&app, "starship.png", false)?;
    assert_eq!(app.input(Input::Tap(1060.0, 1280.0)), Change::Partial);
    assert_eq!(app.game.event(), Event::Deflected);
    shot(&app, "starship-playing.png", true)?;
    assert_eq!(app.input(Input::Tap(985.0, 1545.0)), Change::Full);
    assert_eq!(app.input(Input::Tap(500.0, 860.0)), Change::Full);
    assert_eq!(app.input(Input::Tap(1120.0, 92.0)), Change::Exit);
    Ok(())
}
