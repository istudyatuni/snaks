use std::time::{Duration, Instant};

use anyhow::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::{border, Marker},
    widgets::{
        block::{Position, Title},
        Block, Widget,
    },
};

use lib::{CoordType, Game, GameEvent, GameStatus, MoveTo, Pos};

use crate::{
    achive::{achivements2map, read_achivements, save_achivement, Achivement, AchivementMap},
    difficulty::*,
};

mod render;

const FPS_CONVERT: u64 = 1000 * 1000;
// interesting, dur2fps(fps(60)) == 62
pub const fn fps(fps: u64) -> Duration {
    Duration::from_micros(FPS_CONVERT / fps)
}
pub const fn dur2fps(dur: Duration) -> u64 {
    FPS_CONVERT / dur.as_micros() as u64
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
    SelectDifficulty {
        was_paused: bool,
    },
    Exit,
}

impl App {
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
            KeyCode::Char('d') => self.set_select_difficulty(),
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
        matches!(self.state, AppState::SelectDifficulty { .. })
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
        if let AppState::SelectDifficulty { was_paused: false } = self.state {
            self.unpause();
        }
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
        self.achivements_map = achivements2map(&self.achivements);
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
        self.state = AppState::SelectDifficulty {
            was_paused: self.paused,
        };
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
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use crate::strings::tr::widgets::app as tr;

        let title = Title::from(format!(" {} ", tr::title).bold());
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
