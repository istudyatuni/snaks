use ratatui::{
    style::Stylize,
    widgets::{Block, Paragraph, Widget},
};

use crate::difficulty::{DifficultyKind, DIFFICULTIES};

#[derive(Debug)]
pub struct DifficultySelect {
    pub difficulty: DifficultyKind,
    pub difficulty_changed: bool,
}

impl Widget for DifficultySelect {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut line = vec!["Select difficulty".bold(), ":".into()];
        for d in DIFFICULTIES {
            line.push(" ".into());
            if d == self.difficulty {
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
                self.difficulty.to_string().blue(),
            ]
            .into(),
            vec!["Press ".into(), "d".blue(), " to cancel".into()].into(),
        ];
        if self.difficulty_changed {
            text.extend_from_slice(&["".into(), "Game will restart".into()]);
        }
        Paragraph::new(text).block(Block::new()).render(area, buf)
    }
}
