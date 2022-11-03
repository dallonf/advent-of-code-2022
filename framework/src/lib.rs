pub mod draw_utils;

use std::sync::Mutex;

pub use anyhow;
pub use ggez;
pub use itertools;
pub use tap;
pub use lazy_static;

use ggez::{graphics::Canvas, GameError};

pub struct DrawContext {
    pub draw_fn: Mutex<Option<DrawFn>>,
}

pub type DrawFn = Box<dyn Fn(&mut Canvas, &mut ggez::Context) -> Result<(), GameError> + Send>;

impl DrawContext {
    pub fn set_draw_fn(&self, draw_fn: DrawFn) {
        let mut lock = self.draw_fn.lock().unwrap();
        *lock = Some(Box::new(draw_fn));
    }
}
