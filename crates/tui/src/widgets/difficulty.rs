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
        use crate::strings::tr::widgets::difficulty as tr;

        let mut line = vec![tr::select.bold(), ":".into()];
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
                format!("{} ", tr::press).into(),
                "Enter".blue(),
                format!(" {} ", tr::to_select).into(),
                self.difficulty.to_string().blue(),
            ]
            .into(),
            vec![
                format!("{} ", tr::press).into(),
                "d".blue(),
                format!(" {}", tr::to_cancel).into(),
            ]
            .into(),
        ];
        if self.difficulty_changed {
            text.extend_from_slice(&["".into(), tr::game_restart.into()]);
        }
        Paragraph::new(text).block(Block::new()).render(area, buf)
    }
}
