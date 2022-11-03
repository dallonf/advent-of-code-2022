use macroquad::prelude::*;
use std::{
    sync::{Mutex, TryLockError},
    thread,
};
use tap::prelude::*;
use test_algo::PartTwoProgress;

mod draw_utils;
mod test_algo;

pub struct DrawContext {
    draw_fn: Mutex<Option<DrawFn>>,
}

type DrawFn = Box<dyn Fn() -> () + Send>;

impl DrawContext {
    pub fn set_draw_fn(&self, draw_fn: DrawFn) {
        let mut lock = self.draw_fn.lock().unwrap();
        *lock = Some(Box::new(draw_fn));
    }
}

#[macroquad::main("Advent of Code 2022")]
async fn main() {
    let draw_ctx: &'static DrawContext = Box::new(DrawContext {
        draw_fn: Mutex::new(None),
    })
    .pipe(|it| Box::leak(it));
    let mut current_draw_fn: DrawFn = Box::new(|| ());

    thread::spawn(move || {
        let mut progress = test_algo::viz::PartTwoViz::new(&draw_ctx);
        test_algo::part_two(&mut progress);
    });

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
