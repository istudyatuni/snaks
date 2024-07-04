mod achive;
mod app;
mod difficulty;
mod snake;
mod tui;

const PKG_NAME: &str = "snaks";

fn main() -> anyhow::Result<()> {
    let res = app::App::default().run(&mut tui::init()?);
    tui::restore()?;
    res
}
