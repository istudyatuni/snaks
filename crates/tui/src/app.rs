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

use lib::{Game, GameStatus, MoveTo, Pos};

use crate::snake::SnakeField;

const fn fps(fps: u64) -> Duration {
    Duration::from_millis(1000 / fps)
}
const FPS60: Duration = fps(60);

const SNAKE_FPS: Duration = fps(10);

const DRAW_MARKER: Marker = Marker::Block;

/// Scale frame size to number of cells
const SCALE_SIZE: (f64, f64) = (4.1, 2.2);

#[derive(Debug, Default)]
pub struct App {
    game: Game,
    block_size: Pos,
    game_size: Pos,
    state: AppState,
    paused: bool,
    debug: bool,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum AppState {
    #[default]
    Play,
    Exit,
}

impl App {
    pub fn run(&mut self, term: &mut crate::tui::Tui) -> Result<()> {
        let mut snake_tick = Instant::now();

        while !self.exited() {
            term.draw(|f| {
                let size = f.size();
                let size = Pos::new(size.width as u32, size.height as u32);
                // resize field
                if size != self.block_size {
                    self.block_size = size;
                    self.restart();
                }
                self.render_frame(f);
            })?;

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

        let contraints = [25, 50];
        let contraints = contraints.map(Constraint::Percentage);
        let outer = Layout::horizontal(contraints).split(frame.size());
        let field = Layout::vertical(contraints).split(outer[1]);

        let contraints = if self.debug { [10, 90] } else { [75, 25] };
        let contraints = contraints.map(Constraint::Percentage);
        let stats = Layout::vertical(contraints).split(field[0]);

        frame.render_widget(self.stats_page(), stats[1]);
        frame.render_widget(self.field_canvas(field[1]), field[1]);
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
        match event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('r') => self.restart(),
            KeyCode::Esc => self.toggle_pause(),
            KeyCode::Char('d') => self.toggle_debug(),
            _ => {}
        }

        if self.paused {
            return;
        }

        match event.code {
            KeyCode::Left => self.move_snake(MoveTo::Left),
            KeyCode::Right => self.move_snake(MoveTo::Right),
            KeyCode::Up => self.move_snake(MoveTo::Up),
            KeyCode::Down => self.move_snake(MoveTo::Down),
            _ => {}
        }
    }

    fn exited(&self) -> bool {
        self.state == AppState::Exit
    }

    fn exit(&mut self) {
        self.state = AppState::Exit;
    }
    fn toggle_pause(&mut self) {
        self.paused = !self.paused
    }
    fn toggle_debug(&mut self) {
        self.debug = !self.debug
    }
    fn unpause(&mut self) {
        self.paused = false;
    }
    fn restart(&mut self) {
        self.scale_game_field();
        self.game = Game::new(self.game_size);
        self.unpause();
    }
    fn scale_game_field(&mut self) {
        let (x, y) = self.block_size.into();
        self.game_size = Pos::new(
            (x as f64 / SCALE_SIZE.0) as u32,
            (y as f64 / SCALE_SIZE.1) as u32,
        );
    }
    fn auto_move_snake(&self) {
        self.game.auto_move();
    }
    fn move_snake(&self, to: MoveTo) {
        self.game.rotate_to(to);
    }

    fn field_canvas(&self, size: Rect) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().cyan())
            .x_bounds([0.0, size.width as f64])
            .y_bounds([0.0, size.height as f64])
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
                GameStatus::Play => unreachable!(),
            };
            // todo: render this on top of field_canvas
            text.push(msg.into());
        }
        if self.paused {
            text.push("Pause".into());
        }
        if self.debug {
            if !self.paused && stats.status == GameStatus::Play {
                text.push("".into());
            }
            text.push(format!("Block size (in px): {}", self.block_size).into());
            text.push(format!("Field size: {}", self.game_size).into());
            text.push(format!("Food at: {}", self.game.food()).into());
            text.push(format!("Snake head at: {}", self.game.snake().last().unwrap()).into());
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
        if self.debug {
            make_keybind("Debug", "d", true);
        }
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
