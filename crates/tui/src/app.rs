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
        Block, Padding, Paragraph, Widget,
    },
    Frame,
};

use lib::{CoordType, Game, GameEvent, GameStatus, MoveTo, Pos};

use crate::{
    achive::{achivements2map, read_achivements, AchivementMap},
    snake::SnakeField,
};
use crate::{
    achive::{save_achivement, Achivement},
    difficulty::*,
};

// interesting, dur2fps(fps(60)) == 62
pub const fn fps(fps: u64) -> Duration {
    Duration::from_millis(1000 / fps)
}
pub const fn dur2fps(dur: Duration) -> u64 {
    1000 / dur.as_millis() as u64
}
const FPS20: Duration = fps(20);
const FPS60: Duration = fps(60);

const DEFAULT_UI_FPS: Duration = FPS20;
const DEFAULT_EVENT_FPS: Duration = FPS60;

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

    ui_fps: Duration,
    event_fps: Duration,
    paused: bool,

    show_achivements_grouped: bool,
    achivements: Vec<Achivement>,
    achivements_map: AchivementMap,

    debug: bool,
    debug_info: Debug,
    error: Option<Result<()>>,
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
        self.read_achivement();
        self.update_fps();

        let mut global_tick = Instant::now();
        let mut snake_tick = Instant::now();

        while !self.exited() {
            self.handle_error()?;

            if global_tick.elapsed() < self.ui_fps {
                std::thread::sleep(self.ui_fps - global_tick.elapsed());
            }
            global_tick = Instant::now();
            term.draw(|f| {
                let size = f.size();
                let size = Pos::new(size.width as CoordType, size.height as CoordType);
                // resize field
                if size != self.block_size {
                    self.block_size = size;
                    self.restart();
                }
                self.render_frame(f);
            })?;

            if let Some(e) = self.game.last_event() {
                if e == GameEvent::FoodEat {
                    self.update_achivement();
                    self.handle_error()?;
                    self.game.forgot_event(e);
                }
            }

            if snake_tick.elapsed() > self.difficulty.fps.duration() {
                self.handle_events()?;

                if !self.paused {
                    self.move_snake();
                }
                snake_tick = Instant::now();
            }
        }

        Ok(())
    }

    // -------- handle events --------

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(self.event_fps)? {
            match event::read()? {
                Event::Key(e) if e.kind == KeyEventKind::Press => self.handle_key_event(e),
                _ => {}
            }
        }
        Ok(())
    }
    #[allow(clippy::single_match)]
    fn handle_key_event(&mut self, event: KeyEvent) {
        // common keys
        match event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('r') => self.restart(),
            KeyCode::F(3) => self.toggle_debug(),
            _ => {}
        }

        // keys for selecting difficulty
        if self.selecting_difficulty() {
            match event.code {
                KeyCode::Char('s') => self.select_difficulty(DifficultyKind::Secret),
                KeyCode::Left => self.select_difficulty(self.difficulty.kind.prev()),
                KeyCode::Right => self.select_difficulty(self.difficulty.kind.next()),
                KeyCode::Char('d') => self.undo_difficulty(),
                KeyCode::Enter => self.submit_difficulty(),
                _ => {}
            }
            return;
        }

        // keys when playing
        match event.code {
            KeyCode::Char('d') => {
                self.toggle_pause();
                self.set_select_difficulty();
            }
            KeyCode::Char('a') => self.toggle_achivements_grouped(),
            _ => {}
        }

        if self.game_ended() {
            return;
        }

        // keys when playing except fail/win
        match event.code {
            KeyCode::Esc => self.toggle_pause(),
            _ => {}
        }

        if self.paused {
            return;
        }

        // keys for snake rotate
        match event.code {
            KeyCode::Left => self.rotate_snake(MoveTo::Left),
            KeyCode::Right => self.rotate_snake(MoveTo::Right),
            KeyCode::Up => self.rotate_snake(MoveTo::Up),
            KeyCode::Down => self.rotate_snake(MoveTo::Down),
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
    fn game_ended(&self) -> bool {
        self.game.stats().status != GameStatus::Play
    }
    fn selecting_difficulty(&self) -> bool {
        self.state == AppState::SelectDifficulty
    }
    fn difficulty_changed(&self) -> bool {
        self.difficulty.prev != self.difficulty.kind
    }
    fn handle_error(&mut self) -> Result<()> {
        self.error.take().transpose().map(|_| ())
    }

    // -------- set values --------

    fn select_difficulty(&mut self, d: DifficultyKind) {
        self.difficulty.kind = d;
    }
    fn reset_difficulty(&mut self) {
        self.difficulty.kind = self.difficulty.prev;
    }
    fn undo_difficulty(&mut self) {
        self.reset_difficulty();
        self.unpause();
        self.reset_app_state();
    }
    fn submit_difficulty(&mut self) {
        if !self.difficulty_changed() {
            self.unpause();
            self.reset_app_state();
            return;
        }
        self.difficulty.prev = self.difficulty.kind;
        self.difficulty.update_fps();
        self.update_fps();
        self.restart();
    }
    fn update_fps(&mut self) {
        let fps = self.difficulty.fps.duration();
        self.ui_fps = std::cmp::min(DEFAULT_UI_FPS, fps);
        self.event_fps = std::cmp::min(DEFAULT_EVENT_FPS, fps);
        self.debug_info.fps = format!(
            "{} / {} / {}",
            self.difficulty.fps,
            dur2fps(self.ui_fps),
            dur2fps(self.event_fps),
        );
    }
    fn update_achivement(&mut self) {
        self.save_achivement();
        self.read_achivement();
    }
    fn save_achivement(&mut self) {
        if let e @ Err(_) = save_achivement(Achivement {
            username: whoami::username(),
            difficulty: self.difficulty.kind,
            score: self.game.stats().score,
        }) {
            self.error = Some(e);
        }
    }
    fn read_achivement(&mut self) {
        match read_achivements() {
            Ok(a) => self.achivements = a,
            e @ Err(_) => {
                self.error = Some(e.map(|_| ()));
                return;
            }
        }
        self.achivements_map = achivements2map(self.achivements.clone());
    }

    // -------- set game states --------

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
    fn toggle_achivements_grouped(&mut self) {
        self.show_achivements_grouped = !self.show_achivements_grouped;
    }

    // -------- set game values --------

    fn restart(&mut self) {
        self.scale_game_field();
        self.game = Game::new(self.game_size);
        self.reset_difficulty();
        self.unpause();
        self.reset_app_state();
    }
    fn scale_game_field(&mut self) {
        let (x, y) = self.block_size.into();
        self.game_size = Pos::new(
            (x as f64 / SCALE_SIZE.0) as CoordType,
            (y as f64 / SCALE_SIZE.1) as CoordType,
        );
    }
    fn move_snake(&self) {
        self.game.move_snake();
    }
    fn rotate_snake(&self, to: MoveTo) {
        self.game.rotate_to(to);
    }

    // -------- render ui --------

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());

        let contraints = [25, 50, 25].map(Constraint::Percentage);
        let outer = Layout::horizontal(contraints).split(frame.size());
        let debug = Layout::vertical(contraints).split(outer[0]);
        let field = Layout::vertical(contraints).split(outer[1]);

        let contraints = [26, 50].map(Constraint::Percentage);
        let achivements = Layout::vertical(contraints).split(outer[2]);

        let contraints = [65, 35].map(Constraint::Percentage);
        let stats = Layout::vertical(contraints).split(field[0]);

        let contraints = [30, 70].map(Constraint::Percentage);
        let debug = Layout::horizontal(contraints).split(debug[1]);

        frame.render_widget(self.info_block(), stats[1]);
        if self.debug {
            frame.render_widget(self.debug_block(), debug[1]);
        }
        if self.selecting_difficulty() {
            frame.render_widget(self.difficulty_select(), field[1]);
        } else {
            frame.render_widget(self.field_canvas(field[1]), field[1]);
            frame.render_widget(self.achivements_block(), achivements[1]);
        }
    }
    /// Canvas with snake field
    fn field_canvas(&self, size: Rect) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().cyan())
            .x_bounds([0.0, size.width as f64])
            .y_bounds([0.0, size.height as f64])
            .marker(DRAW_MARKER)
            .paint(|ctx| ctx.draw(&SnakeField::new(self.game.snake(), self.game.food())))
    }
    /// Game info + debug info
    fn info_block(&self) -> impl Widget + '_ {
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
        if self.game_ended() {
            let msg = match stats.status {
                GameStatus::Fail => "Game Over".red(),
                GameStatus::Win => "Win".green(),
                GameStatus::Play => unreachable!(),
            };
            // todo: render this on top of field_canvas
            text.push(msg.into());
        }
        let show_pause = self.paused && !self.selecting_difficulty();
        if show_pause {
            text.push("Pause".yellow().into());
        }
        Paragraph::new(text).block(Block::new())
    }
    fn debug_block(&self) -> impl Widget + '_ {
        let text = vec![
            format!("Block size: {}", self.block_size).into(),
            format!("Field size: {}", self.game_size).into(),
            format!("Food: {}", self.game.food()).into(),
            format!("Snake head: {}", self.game.head()).into(),
            "FPS (snake / ui / event):".into(),
            format!("  {}", self.debug_info.fps).into(),
            format!("Snake direction: {}", self.game.direction()).into(),
        ];
        Paragraph::new(text).block(Block::new().padding(Padding::uniform(1)))
    }
    /// Block with difficulty select
    fn difficulty_select(&self) -> impl Widget + '_ {
        let mut line = vec!["Select difficulty".bold(), ":".into()];
        for d in DIFFICULTIES {
            line.push(" ".into());
            if d == self.difficulty.kind {
                line.push(d.to_string().blue());
            } else {
                line.push(d.to_string().into());
            }
        }
        let mut text: Vec<_> = vec![
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
        if self.difficulty_changed() {
            text.extend_from_slice(&["".into(), "Game will restart".into()]);
        }
        Paragraph::new(text).block(Block::new())
    }
    /// Block with achivements. Only for current difficulty
    fn achivements_block(&self) -> impl Widget + '_ {
        let achivements: Vec<_> = if self.show_achivements_grouped {
            self.achivements_grouped()
        } else {
            self.achivements_by_user()
        };

        let mut text: Vec<_> = if self.show_achivements_grouped {
            vec![vec![
                "Achivements on ".into(),
                self.difficulty.kind.to_string().blue(),
            ]
            .into()]
        } else {
            vec![vec!["Achivements".into()].into()]
        };
        text.push("".into());
        text.extend_from_slice(&achivements);

        Paragraph::new(text).block(Block::new().padding(Padding::uniform(1)))
    }

    // -------- render utilities --------

    fn keybind_help(&self) -> Line<'_> {
        let mut instructions = vec![];
        let mut show_keybind = |name: &'static str, key: &'static str, sep| {
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
        if self.playing() && !self.paused {
            show_keybind("Move", "← ↑ → ↓", true);
        } else if self.selecting_difficulty() {
            show_keybind("Select", "← →", true);
            show_keybind("Submit", "Enter", true);
            show_keybind("Cancel", "d", true);
        }
        if self.playing() && !self.game_ended() {
            if !self.paused {
                show_keybind("Pause", "Esc", true);
            } else if self.paused {
                show_keybind("Resume", "Esc", true);
            }
        }
        if !self.selecting_difficulty() {
            if self.show_achivements_grouped {
                show_keybind("Show achivements by user", "a", true);
            } else {
                show_keybind("Show achivements summary", "a", true);
            }
            show_keybind("Difficulty", "d", true);
        }
        show_keybind("Restart", "r", true);
        if self.debug {
            show_keybind("Debug", "F3", true);
        }
        show_keybind("Quit", "q", false);
        Line::from(instructions)
    }
    /// Group achivements by user
    fn achivements_by_user(&self) -> Vec<Line<'_>> {
        self.achivements_map
            .iter()
            .flat_map(|(user, a)| {
                let a: Vec<_> = a
                    .iter()
                    .map(|a| {
                        vec![
                            "  ".into(),
                            a.difficulty.to_string().blue(),
                            " ".into(),
                            a.score.to_string().into(),
                        ]
                        .into()
                    })
                    .collect();
                let mut res = vec![vec![user.clone().blue()].into()];
                res.extend_from_slice(&a);
                res
            })
            .collect()
    }
    /// Show all achivements on current difficulty
    fn achivements_grouped(&self) -> Vec<Line<'_>> {
        self.achivements
            .iter()
            .filter(|a| a.difficulty == self.difficulty.kind)
            .map(|a| {
                vec![
                    a.username.to_owned().blue(),
                    " ".into(),
                    a.score.to_string().into(),
                ]
                .into()
            })
            .collect()
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Title::from(" Snake Game ".bold());
        let help = Title::from(self.keybind_help());
        Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(help.alignment(Alignment::Center).position(Position::Bottom))
            .border_set(border::THICK)
            .render(area, buf);
    }
}

#[derive(Debug, Default)]
struct Debug {
    fps: String,
}
