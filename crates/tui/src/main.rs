use std::time::Duration;

mod app;
mod tui;

const fn fps(fps: u64) -> Duration {
    Duration::from_millis(1000 / fps)
}
const FPS60: Duration = fps(60);

fn main() -> anyhow::Result<()> {
    let mut term = tui::init()?;
    // term.clear()?;

    let res = app::App::default().run(&mut term);
    tui::restore()?;
    res
}
