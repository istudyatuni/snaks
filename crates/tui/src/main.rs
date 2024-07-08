mod achive;
mod app;
mod difficulty;
mod strings;
mod tui;
mod widgets;

const PKG_NAME: &str = "snaks";

fn main() -> anyhow::Result<()> {
    let res = app::App::default().run(&mut tui::init()?, whoami::username());
    tui::restore()?;
    res
}
