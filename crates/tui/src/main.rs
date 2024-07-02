mod app;
mod snake;
mod tui;

fn main() -> anyhow::Result<()> {
    let res = app::App::new().run(&mut tui::init()?);
    tui::restore()?;
    res
}
