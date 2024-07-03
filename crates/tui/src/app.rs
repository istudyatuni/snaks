use std::{
    fmt::Display,
    time::{Duration, Instant},
};

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

// interesting, dur2fps(fps(60)) == 62
const fn fps(fps: u64) -> Duration {
    Duration::from_millis(1000 / fps)
}
const fn dur2fps(dur: Duration) -> u64 {
    1000 / dur.as_millis() as u64
}
const FPS60: Duration = fps(60);

const DRAW_MARKER: Marker = Marker::Block;

/// Scale frame size to number of cells
const SCALE_SIZE: (f64, f64) = (4.1, 2.2);

#[derive(Debug, Default)]
pub struct App {
    game: Game,
    block_size: Pos,
    game_size: Pos,
    state: AppState,
    difficulty: Difficulty,
    paused: bool,
    debug: bool,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum AppState {
    #[default]
    Play,
    SelectDifficulty,
    Exit,
}

impl App {
    // -------- run --------

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

            if snake_tick.elapsed() > self.difficulty.fps.0 {
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

        let contraints = if self.debug { [10, 90] } else { [65, 35] };
        let contraints = contraints.map(Constraint::Percentage);
        let stats = Layout::vertical(contraints).split(field[0]);

        frame.render_widget(self.stats_page(), stats[1]);
        if self.selecting_difficulty() {
            frame.render_widget(self.difficulty_select(), field[1]);
        } else {
            frame.render_widget(self.field_canvas(field[1]), field[1]);
        }
    }

    // -------- handle events --------

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
            KeyCode::F(3) => self.toggle_debug(),
            _ => {}
        }

        if self.selecting_difficulty() {
            match event.code {
                KeyCode::Char(c @ '1'..='6') => {
                    self.select_difficulty(DifficultyKind::from_number(c))
                }
                KeyCode::Char('d') => self.undo_difficulty(),
                KeyCode::Enter => self.submit_difficulty(),
                _ => {}
            }
            return;
        }

        match event.code {
            KeyCode::Char('d') => {
                self.toggle_pause();
                self.set_select_difficulty();
            }
            KeyCode::Esc => self.toggle_pause(),
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

    // -------- get values --------

    fn exited(&self) -> bool {
        self.state == AppState::Exit
    }
    fn playing(&self) -> bool {
        self.state == AppState::Play
    }
    fn selecting_difficulty(&self) -> bool {
        self.state == AppState::SelectDifficulty
    }

    // -------- set values --------

    fn select_difficulty(&mut self, d: DifficultyKind) {
        self.difficulty.kind = d;
    }
    fn undo_difficulty(&mut self) {
        self.difficulty.kind = self.difficulty.prev;
        self.unpause();
        self.reset_app_state();
    }
    fn submit_difficulty(&mut self) {
        self.difficulty.prev = self.difficulty.kind;
        self.difficulty.update_fps();
        self.restart();
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
    fn set_select_difficulty(&mut self) {
        self.state = AppState::SelectDifficulty;
        self.pause();
    }
    fn pause(&mut self) {
        self.paused = true;
    }
    fn unpause(&mut self) {
        self.paused = false;
    }
    fn reset_app_state(&mut self) {
        self.state = AppState::Play;
    }

    // -------- set game values --------

    fn restart(&mut self) {
        self.scale_game_field();
        self.game = Game::new(self.game_size);
        self.unpause();
        self.reset_app_state();
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

    // -------- render ui --------

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

        let difficulty = if self.selecting_difficulty() {
            self.difficulty.prev
        } else {
            self.difficulty.kind
        };
        let mut text = vec![
            vec!["Score ".blue(), format!("{}", stats.score).into()].into(),
            vec!["Difficulty ".blue(), format!("{difficulty}").into()].into(),
        ];
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
            text.push("Pause".blue().into());
        }
        if self.debug {
            if !self.paused && stats.status == GameStatus::Play {
                text.push("".into());
            }
            text.push(format!("Block size: {}", self.block_size).into());
            text.push(format!("Field size: {}", self.game_size).into());
            text.push(format!("Food: {}", self.game.food()).into());
            text.push(format!("Snake head: {}", self.game.snake().last().unwrap()).into());
            text.push(format!("Snake FPS: {}", self.difficulty.fps).into());
            text.push(format!("Snake direction: {}", self.game.direction()).into());
        }
        Paragraph::new(text).block(Block::new())
    }
    fn difficulty_select(&self) -> impl Widget + '_ {
        let mut line = vec!["Select difficulty".bold(), ":".into()];
        for (i, &d) in DIFFICULTIES.iter().enumerate() {
            line.push(format!(" [{}] ", i + 1).into());
            if d == self.difficulty.kind {
                line.push(d.to_string().blue());
            } else {
                line.push(d.to_string().into());
            }
        }
        let text: Vec<_> = vec![
            "".into(),
            line.into(),
            "".into(),
            vec![
                "Press ".into(),
                "Enter".blue(),
                " to select ".into(),
                self.difficulty.kind.to_string().blue(),
            ]
            .into(),
            vec!["Press ".into(), "d".blue(), " to cancel".into()].into(),
        ];
        Paragraph::new::<Vec<_>>(text).block(Block::new())
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
        if self.playing() {
            make_keybind("Move", "← ↑ → ↓", true);
        }
        if self.selecting_difficulty() {
            make_keybind("Select", "1 2 3 4 5", true);
            make_keybind("Submit", "Enter", true);
            make_keybind("Cancel", "d", true);
        }
        make_keybind("Restart", "r", true);
        if self.playing() {
            if !self.paused {
                make_keybind("Pause", "Esc", true);
            } else if self.paused {
                make_keybind("Resume", "Esc", true);
            }
        }
        if !self.selecting_difficulty() {
            make_keybind("Difficulty", "d", true);
        }
        if self.debug {
            make_keybind("Debug", "F3", true);
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

#[derive(Debug, Default)]
struct Difficulty {
    prev: DifficultyKind,
    kind: DifficultyKind,
    fps: DifficultyFps,
}

impl Difficulty {
    fn update_fps(&mut self) {
        self.fps = self.kind.to_fps()
    }
}

#[derive(Debug)]
struct DifficultyFps(Duration);

impl Default for DifficultyFps {
    fn default() -> Self {
        DifficultyKind::default().to_fps()
    }
}

impl Display for DifficultyFps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", dur2fps(self.0))
    }
}

const DIFFICULTIES: [DifficultyKind; 5] = [
    DifficultyKind::Easy,
    DifficultyKind::Normal,
    DifficultyKind::Medium,
    DifficultyKind::Hard,
    DifficultyKind::Impossible,
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum DifficultyKind {
    Easy,
    #[default]
    Normal,
    Medium,
    Hard,
    Impossible,
    Secret,
}

impl DifficultyKind {
    fn to_fps(self) -> DifficultyFps {
        let f = match self {
            DifficultyKind::Easy => 5,
            DifficultyKind::Normal => 10,
            DifficultyKind::Medium => 15,
            DifficultyKind::Hard => 30,
            DifficultyKind::Impossible => 60,
            DifficultyKind::Secret => 100,
        };
        DifficultyFps(fps(f))
    }
    fn from_number(n: char) -> Self {
        match n {
            '1' => Self::Easy,
            '2' => Self::Normal,
            '3' => Self::Medium,
            '4' => Self::Hard,
            '5' => Self::Impossible,
            '6' => Self::Secret,
            _ => Self::default(),
        }
    }
}

impl Display for DifficultyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DifficultyKind::Easy => "Easy",
            DifficultyKind::Normal => "Normal",
            DifficultyKind::Medium => "Medium",
            DifficultyKind::Hard => "Hard",
            DifficultyKind::Impossible => "Impossible",
            DifficultyKind::Secret => "Secret",
        };
        f.pad(s)
    }
}
