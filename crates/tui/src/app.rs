use std::time::{Duration, Instant};

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
        Block, Paragraph, Widget,
    },
    Frame,
};

use lib::{Game, GameStatus, MoveTo};

use crate::snake::SnakeField;

const fn fps(fps: u64) -> Duration {
    Duration::from_millis(1000 / fps)
}
const FPS60: Duration = fps(60);

const SNAKE_FPS: Duration = fps(10);

const DRAW_MARKER: Marker = Marker::Block;

#[derive(Debug)]
pub struct App {
    game: Game,
    exited: bool,
    paused: bool,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            game: Self::new_game(),
            exited: false,
            paused: false,
        }
    }
    fn new_game() -> Game {
        Game::new(39.into(), 17.into(), (20, 10).into())
    }
    pub fn run(&mut self, term: &mut crate::tui::Tui) -> Result<()> {
        let mut snake_tick = Instant::now();

        while !self.exited {
            term.draw(|f| self.render_frame(f))?;

            if snake_tick.elapsed() > SNAKE_FPS {
                self.handle_events()?;

                if !self.paused {
                    self.auto_move_snake();
                }
                snake_tick = Instant::now();
            }
        }
        Ok(())
    }
    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());

        let contraints = [Constraint::Percentage(25), Constraint::Percentage(50)];
        let outer = Layout::horizontal(contraints).split(frame.size());
        let field = Layout::vertical(contraints).split(outer[1]);
        let stats = Layout::vertical([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(field[0]);

        frame.render_widget(self.stats_page(), stats[1]);
        frame.render_widget(self.field_canvas(), field[1]);
    }
    fn handle_events(&mut self) -> Result<()> {
        if event::poll(FPS60)? {
            match event::read()? {
                Event::Key(e) if e.kind == KeyEventKind::Press => self.handle_key_event(e),
                _ => {}
            }
        }
        Ok(())
    }
    fn handle_key_event(&mut self, event: KeyEvent) {
        if self.paused {
            match event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Esc => self.unpause(),
                _ => {}
            }
            return;
        }

        match event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('r') => self.restart(),
            KeyCode::Esc => self.pause(),
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
    fn pause(&mut self) {
        self.paused = true;
    }
    fn unpause(&mut self) {
        self.paused = false;
    }
    fn restart(&mut self) {
        self.game = Self::new_game();
        self.unpause();
    }
    fn auto_move_snake(&self) {
        self.game.auto_move();
    }
    fn move_snake(&self, to: MoveTo) {
        self.game.rotate_to(to);
    }

    fn field_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().cyan())
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .marker(DRAW_MARKER)
            .paint(|ctx| ctx.draw(&SnakeField::new(self.game.snake(), self.game.food())))
    }
    fn stats_page(&self) -> impl Widget + '_ {
        let stats = self.game.stats();

        let mut text = vec![vec!["Score ".into(), format!("{}", stats.score).blue()].into()];
        if stats.status != GameStatus::Play {
            let msg = match stats.status {
                GameStatus::Fail => "Game Over".red(),
                GameStatus::Win => "Win".green(),
                _ => unreachable!(),
            };
            // todo: render this on top of field_canvas
            text.push(msg.into());
        }
        if self.paused {
            text.push("Pause".into());
        }
        Paragraph::new(text).block(Block::new())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Title::from(" Snake Game ".bold());

        let mut instructions = vec![];
        let mut make_keybind = |name: &'static str, key: &'static str, sep| {
            const SP: &str = " ";
            const SEP: &str = "|";
            instructions.extend(vec![
                SP.into(),
                name.into(),
                SP.into(),
                key.blue().bold(),
                SP.into(),
            ]);
            if sep {
                instructions.push(SEP.into());
            }
        };
        make_keybind("Move", "← ↑ → ↓", true);
        make_keybind("Restart", "r", true);
        make_keybind("Pause", "Esc", true);
        make_keybind("Quit", "q", false);
        let instructions = Title::from(Line::from(instructions));

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
