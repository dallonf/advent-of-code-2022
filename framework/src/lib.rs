pub mod draw_utils;

use std::sync::Mutex;

use ggez::{graphics::Canvas, Context, GameError};

pub struct DrawContext {
    pub draw_fn: Mutex<Option<DrawFn>>,
}

pub type DrawFn = Box<dyn Fn(&mut Canvas, &mut Context) -> Result<(), GameError> + Send>;

impl DrawContext {
    pub fn set_draw_fn(&self, draw_fn: DrawFn) {
        let mut lock = self.draw_fn.lock().unwrap();
        *lock = Some(Box::new(draw_fn));
    }
}
