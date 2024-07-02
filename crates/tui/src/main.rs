mod app;
mod snake;
mod tui;

fn main() -> anyhow::Result<()> {
    let mut term = tui::init()?;
    // term.clear()?;

    let res = app::App::new().run(&mut term);
    tui::restore()?;
    res
}
