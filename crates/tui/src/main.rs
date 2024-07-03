mod app;
mod difficulty;
mod snake;
mod tui;

fn main() -> anyhow::Result<()> {
    let res = app::App::default().run(&mut tui::init()?);
    tui::restore()?;
    res
}
