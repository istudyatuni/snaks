use anyhow::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Constraint, Layout, Rect},
    style::Stylize,
    symbols::{border, Marker},
    text::Line,
    widgets::{
        block::{Position, Title},
        canvas::Canvas,
        Block, Widget,
    },
    Frame,
};

use lib::{Game, MoveTo};

use crate::snake::SnakeField;

/*const fn fps(fps: u64) -> Duration {
    Duration::from_millis(1000 / fps)
}
const FPS60: Duration = fps(60);

const SNAKE_FPS: Duration = fps(20);*/

const DRAW_MARKER: Marker = Marker::Block;

#[derive(Debug)]
pub struct App {
    game: Game,
    exited: bool,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            game: Game::new(39.into(), 17.into(), (20, 10).into()),
            exited: false,
        }
    }
    pub fn run(&mut self, term: &mut crate::tui::Tui) -> Result<()> {
        // let mut last_tick = Instant::now();

        while !self.exited {
            term.draw(|f| self.render_frame(f))?;
            self.handle_events()?;

            // tickrate only affect state changes, not redraw
            // if last_tick.elapsed() <
        }
        Ok(())
    }
    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());

        let contraints = [Constraint::Percentage(25), Constraint::Percentage(50)];
        let outer = Layout::horizontal(contraints).split(frame.size());
        let inner = Layout::vertical(contraints).split(outer[1]);

        frame.render_widget(self.field_canvas(), inner[1])
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

    fn field_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().cyan())
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .marker(DRAW_MARKER)
            .paint(|ctx| ctx.draw(&SnakeField::new(self.game.snake(), self.game.food())))
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

        Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK)
            .render(area, buf);
    }
}
