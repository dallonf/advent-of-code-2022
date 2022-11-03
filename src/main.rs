use ggez::{
    conf::{WindowMode, WindowSetup},
    graphics::{self, Canvas},
    Context, ContextBuilder, GameError,
};
use std::{
    sync::{Mutex, TryLockError},
    thread,
};
use tap::prelude::*;

mod draw_utils;
mod test_algo;

pub struct DrawContext {
    draw_fn: Mutex<Option<DrawFn>>,
}

type DrawFn = Box<dyn Fn(&mut Canvas, &mut Context) -> Result<(), GameError> + Send>;

impl DrawContext {
    pub fn set_draw_fn(&self, draw_fn: DrawFn) {
        let mut lock = self.draw_fn.lock().unwrap();
        *lock = Some(Box::new(draw_fn));
    }
}

struct AppState {
    current_draw_fn: DrawFn,
    draw_ctx: &'static DrawContext,
}

impl ggez::event::EventHandler<GameError> for AppState {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        match self.draw_ctx.draw_fn.try_lock() {
            Ok(mut draw_fn) => {
                let inner = draw_fn.take();
                if let Some(new_draw_fn) = inner {
                    self.current_draw_fn = new_draw_fn;
                }
            }
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Poisoned(_)) => return Err(GameError::LockError),
        }
        let mut canvas = graphics::Canvas::from_frame(ctx, draw_utils::WHITE);
        (self.current_draw_fn)(&mut canvas, ctx)?;
        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() {
    let conf = ggez::conf::Conf::new();
    let (ctx, event_loop) = ContextBuilder::new("aoc2022", "dallonf")
        .default_conf(conf)
        .window_mode(
            WindowMode::default()
                .resizable(false)
                .dimensions(1366.0, 768.0)
                .resize_on_scale_factor_change(false),
        )
        .window_setup(WindowSetup::default().title("Advent of Code 2022"))
        .build()
        .unwrap();

    let draw_ctx: &'static DrawContext = Box::new(DrawContext {
        draw_fn: Mutex::new(None),
    })
    .pipe(|it| Box::leak(it));
    thread::spawn(move || {
        let mut progress = test_algo::viz::PartTwoViz::new(&draw_ctx);
        test_algo::part_two(&mut progress);
    });

    let initial_state = AppState {
        current_draw_fn: Box::new(|_, _| Ok(())),
        draw_ctx,
    };
    ggez::event::run(ctx, event_loop, initial_state);
}
