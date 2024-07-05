use ratatui::{
    style::Stylize,
    widgets::{Block, Paragraph, Widget},
};

use crate::difficulty::Difficulty;

use lib::{GameStatus, Stats};

#[derive(Debug)]
pub struct Info {
    pub difficulty: Difficulty,
    pub stats: Stats,
    pub game_ended: bool,
    pub show_pause: bool,
}

impl Widget for Info {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut text = vec![
            vec!["Score ".blue(), format!("{}", self.stats.score).into()].into(),
            vec![
                "Difficulty ".blue(),
                format!("{}", self.difficulty.prev).into(),
            ]
            .into(),
        ];
        if self.game_ended {
            let msg = match self.stats.status {
                GameStatus::Fail => "Game Over".red(),
                GameStatus::Win => "Win".green(),
                GameStatus::Play => unreachable!(),
            };
            // todo: render this on top of field_canvas
            text.push(msg.into());
        }
        if self.show_pause {
            text.push("Pause".yellow().into());
        }
        Paragraph::new(text).block(Block::new()).render(area, buf)
    }
}
