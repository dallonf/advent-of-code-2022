use crate::DrawContext;
use ggez::{
    glam::Vec2,
    graphics::{self, DrawParam},
};

use crate::draw_utils;

use super::PartTwoProgress;

#[derive(Clone, Debug)]
struct PartAndFuel {
    initial_mass: i64,
    fuel: Vec<i64>,
}

pub struct PartTwoViz {
    ctx: &'static DrawContext,
    parts: Vec<PartAndFuel>,
}

const PART_HEIGHT: f32 = 48.0;
const TEXT_HEIGHT: f32 = 32.0;
const LINE_HEIGHT: f32 = PART_HEIGHT + TEXT_HEIGHT;

impl PartTwoViz {
    pub fn new(ctx: &'static DrawContext) -> Self {
        PartTwoViz {
            ctx: ctx,
            parts: vec![],
        }
    }
    fn render(&self) {
        let parts = self.parts.clone();
        self.ctx.set_draw_fn(Box::new(move |canvas, _ctx| {
            let mut y_cursor: f32 = 0.0;
            for part in parts.iter() {
                let mut text = graphics::Text::new(part.initial_mass.to_string());
                text.set_scale(16.0);
                canvas.draw(
                    &mut text,
                    DrawParam::default()
                        .dest(Vec2::new(8.0, y_cursor))
                        .color(draw_utils::BLACK),
                );
                y_cursor += LINE_HEIGHT;
            }
            Ok(())
        }))
    }
}

impl PartTwoProgress for PartTwoViz {
    fn new_part(&mut self, mass: i64) {
        self.parts.push(PartAndFuel {
            initial_mass: mass,
            fuel: vec![],
        });
        self.render();
    }

    fn additional_fuel(&mut self, mass: i64) {
        // no-op, todo
    }
}
