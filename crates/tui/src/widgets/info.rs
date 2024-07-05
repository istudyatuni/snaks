use ratatui::{
    style::Stylize,
    widgets::{Block, Paragraph, Widget},
};

use crate::difficulty::Difficulty;

use lib::Stats;

#[derive(Debug)]
pub struct Info {
    pub difficulty: Difficulty,
    pub stats: Stats,
    pub show_pause: bool,
}

impl Widget for Info {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        use crate::strings::tr::widgets::info as tr;

        let mut text = vec![
            vec![
                format!("{} ", tr::score).blue(),
                format!("{}", self.stats.score).into(),
            ]
            .into(),
            vec![
                format!("{} ", tr::difficulty).blue(),
                format!("{}", self.difficulty.prev).into(),
            ]
            .into(),
        ];
        if self.show_pause {
            text.push(tr::pause.yellow().into());
        }
        Paragraph::new(text).block(Block::new()).render(area, buf)
    }
}
