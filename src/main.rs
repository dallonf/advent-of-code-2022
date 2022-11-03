use framework::ggez::{
    self,
    conf::{WindowMode, WindowSetup},
    graphics, ContextBuilder, GameError,
};
use framework::tap::prelude::*;
use framework::{draw_utils, DrawContext, DrawFn};
use std::{
    sync::{Mutex, TryLockError},
    thread,
};

#[hot_lib_reloader::hot_module(dylib = "logic")]
mod hot_logic {
    use framework::ggez::GameError;
    use framework::DrawContext;

    hot_functions_from_file!("logic/src/lib.rs");

    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
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
        let observer = hot_logic::subscribe();
        loop {
            let logic_thread =
                thread::spawn(move || hot_logic::run("test_algo", "part_two", draw_ctx));
            // TODO: figure out how to early-terminate a logic thread
            logic_thread.join().unwrap().unwrap();
            observer.wait_for_reload();
        }
    });

    let initial_state = AppState {
        current_draw_fn: Box::new(|_, _| Ok(())),
        draw_ctx,
    };
    ggez::event::run(ctx, event_loop, initial_state);
}
