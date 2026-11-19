use std::{
    fs,
    path::{Path, PathBuf},
};

use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input};
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

    shot(&app, "starship.png", false)?;
    assert_eq!(app.input(Input::Tap(632.0, 1340.0)), Change::Partial);
    for _ in 0..14 {
        assert_eq!(app.tick(app.deadline().unwrap()), Change::Partial);
    }
    assert_eq!(app.input(Input::Tap(804.0, 1128.0)), Change::Partial);
    shot(&app, "starship-playing.png", true)?;
    assert_eq!(app.input(Input::Tap(632.0, 1340.0)), Change::Partial);
    assert!(app.deadline().is_none());
    shot(&app, "starship-paused.png", false)?;
    assert_eq!(app.input(Input::Tap(1120.0, 92.0)), Change::Exit);
    Ok(())
}
