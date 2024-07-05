use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::{
    achive::{Achivement, AchivementMap},
    difficulty::DifficultyKind,
};

#[derive(Debug)]
pub struct Achivements<'ach> {
    pub difficulty: DifficultyKind,
    pub show_achivements_grouped: bool,
    pub achivements: &'ach [Achivement],
    pub achivements_map: &'ach AchivementMap,
}

impl<'ach> Widget for Achivements<'ach> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let achivements: Vec<_> = if self.show_achivements_grouped {
            self.achivements_grouped()
        } else {
            self.achivements_by_user()
        };

        let mut text: Vec<_> = if self.show_achivements_grouped {
            vec![vec!["Achivements on ".into(), self.difficulty.to_string().blue()].into()]
        } else {
            vec![vec!["Achivements".into()].into()]
        };
        text.push("".into());
        text.extend_from_slice(&achivements);

        Paragraph::new(text)
            .block(Block::new().padding(Padding::uniform(1)))
            .render(area, buf)
    }
}

impl<'ach> Achivements<'ach> {
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
            .filter(|a| a.difficulty == self.difficulty)
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
