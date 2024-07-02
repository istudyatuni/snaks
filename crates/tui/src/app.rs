use anyhow::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget,
    },
    Frame,
};

use lib::{Game, MoveTo};

#[derive(Debug, Default)]
pub struct App {
    game: Game,
    exited: bool,
}

impl App {
    pub fn run(&mut self, term: &mut crate::tui::Tui) -> Result<()> {
        while !self.exited {
            term.draw(|f| self.render_frame(f))?;
            self.handle_events()?;
        }
        Ok(())
    }
    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(e) if e.kind == KeyEventKind::Press => self.handle_key_event(e),
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.move_snake(MoveTo::Left),
            KeyCode::Right => self.move_snake(MoveTo::Right),
            KeyCode::Up => self.move_snake(MoveTo::Up),
            KeyCode::Down => self.move_snake(MoveTo::Down),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exited = true;
    }
    fn move_snake(&self, to: MoveTo) {
        self.game.move_to(to);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Title::from(" Snake Game ".bold());
        let instructions = Title::from(Line::from(vec![
            " Move ".into(),
            "← ↑ → ↓".blue().bold(),
            " | ".into(),
            "Quit".into(),
            " q ".blue().bold(),
            // " Help ".into(),
            // "h".blue().bold(),
        ]));

        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let text = Text::from(vec![Line::from(vec!["value".into()])]);

        Paragraph::new(text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
