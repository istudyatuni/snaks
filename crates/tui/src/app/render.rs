use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{canvas::Canvas, Block, Padding, Paragraph, Widget},
    Frame,
};

use crate::{strings::tr, widgets};

use super::{App, DRAW_MARKER};

impl App {
    pub(super) fn render_frame(&self, frame: &mut Frame) {
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
            .paint(|ctx| {
                ctx.draw(&widgets::SnakeField::new(
                    self.game.snake(),
                    self.game.food(),
                ))
            })
    }
    /// Game info + debug info
    fn info_block(&self) -> impl Widget + '_ {
        widgets::Info {
            difficulty: self.difficulty.clone(),
            stats: self.game.stats(),
            game_ended: self.game_ended(),
            show_pause: self.paused && !self.selecting_difficulty(),
        }
    }
    fn debug_block(&self) -> impl Widget + '_ {
    	use tr::widgets::debug as tr;

        let text = vec![
            format!("{}: {}", tr::block_size, self.block_size).into(),
            format!("{}: {}", tr::field_size, self.game_size).into(),
            format!("{}: {}", tr::food, self.game.food()).into(),
            format!("{}: {}", tr::snake_head, self.game.head()).into(),
            format!("{}:", tr::fps).into(),
            format!("  {}", self.debug_info.fps).into(),
            format!("{}: {}", tr::snake_direction, self.game.direction()).into(),
        ];
        Paragraph::new(text).block(Block::new().padding(Padding::uniform(1)))
    }
    /// Block with difficulty select
    fn difficulty_select(&self) -> impl Widget + '_ {
        widgets::DifficultySelect {
            difficulty: self.difficulty.kind,
            difficulty_changed: self.difficulty_changed(),
        }
    }
    /// Block with achivements. Only for current difficulty
    fn achivements_block(&self) -> impl Widget + '_ {
        widgets::Achivements {
            difficulty: self.difficulty.kind,
            show_achivements_grouped: self.show_achivements_grouped,
            achivements: &self.achivements,
            achivements_map: &self.achivements_map,
        }
    }

    // -------- render utilities --------

    pub(super) fn keybind_help(&self) -> Line<'_> {
    	use tr::keybind as tr;

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
            show_keybind(tr::r#move, "← ↑ → ↓", true);
        } else if self.selecting_difficulty() {
            show_keybind(tr::select, "← →", true);
            show_keybind(tr::submit, "Enter", true);
            show_keybind(tr::cancel, "d", true);
        }
        if self.playing() && !self.game_ended() {
            if !self.paused {
                show_keybind(tr::pause, "Esc", true);
            } else if self.paused {
                show_keybind(tr::resume, "Esc", true);
            }
        }
        if !self.selecting_difficulty() {
            if self.show_achivements_grouped {
                show_keybind(tr::achivements_by_user, "a", true);
            } else {
                show_keybind(tr::achivements_summary, "a", true);
            }
            show_keybind(tr::difficulty, "d", true);
        }
        show_keybind(tr::restart, "r", true);
        if self.debug {
            show_keybind(tr::debug, "F3", true);
        }
        show_keybind(tr::quit, "q", false);
        Line::from(instructions)
    }
}
