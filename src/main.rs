use macroquad::prelude::*;
use std::sync::{Mutex, TryLockError};
use tap::prelude::*;

pub struct DrawContext {
    draw_fn: Mutex<Option<DrawFn>>,
}

type DrawFn = Box<dyn FnMut() -> ()>;

impl DrawContext {
    pub fn set_draw_fn(&self, draw_fn: fn() -> ()) {
        let mut lock = self.draw_fn.lock().unwrap();
        *lock = Some(Box::new(draw_fn));
    }
}

#[macroquad::main("Advent of Code 2022")]
async fn main() {
    let draw_ctx = Box::new(DrawContext {
        draw_fn: Mutex::new(None),
    })
    .pipe(|it| Box::leak(it));
    let mut current_draw_fn: DrawFn = Box::new(|| ());
    loop {
        match draw_ctx.draw_fn.try_lock() {
            Ok(mut draw_fn) => {
                let inner = draw_fn.take();
                if let Some(new_draw_fn) = inner {
                    current_draw_fn = new_draw_fn;
                }
            }
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Poisoned(err)) => panic!("{}", err),
        }
        current_draw_fn();
        next_frame().await
    }
}
