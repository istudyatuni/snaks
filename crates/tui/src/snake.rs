use lib::Pos;
use ratatui::{
    style::Color,
    widgets::canvas::{Painter, Shape},
};

#[derive(Debug)]
pub struct SnakeField {
    snake: Vec<Pos>,
    food: Pos,
    snake_color: Color,
    food_color: Color,
}

impl SnakeField {
    pub fn new(snake: Vec<Pos>, food: Pos) -> Self {
        Self {
            snake,
            food,
            snake_color: Color::Green,
            food_color: Color::Red,
        }
    }
}

impl Shape for SnakeField {
    fn draw(&self, painter: &mut Painter) {
        let mut paint_point = |pos: Pos, color| {
            let (x, y) = pos.into();
            let x = x * 2;
            painter.paint(x, y, color);
            painter.paint(x + 1, y, color);
        };

        paint_point(self.food, self.food_color);

        for &pos in &self.snake {
            paint_point(pos, self.snake_color);
        }
    }
}
