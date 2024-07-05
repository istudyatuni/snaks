use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style, Styled},
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug)]
pub struct Finish {
    pub state: Option<FinishState>,
}

#[derive(Debug)]
pub enum FinishState {
    Fail,
    Win,
}

impl Widget for Finish {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        use crate::strings::tr::widgets::finish as tr;

        let Some(state) = self.state else { return };

        const W: u16 = 25;
        const H: u16 = 5;
        const W2: u16 = W / 2 + 1;
        const H2: u16 = H / 2 + 1;
        let rect_fit = Rect::new(area.x - W2, area.y - H2, W, H);

        let (text, color) = match state {
            FinishState::Fail => (tr::fail, Color::Red),
            FinishState::Win => (tr::win, Color::Green),
        };
        let style = Style::new().fg(color);

        let text: Vec<_> = vec![vec![].into(), vec![text.set_style(style)].into()];
        Paragraph::new(text)
            .block(Block::bordered().border_style(style))
            .alignment(Alignment::Center)
            .render(area.clamp(rect_fit), buf)
    }
}
