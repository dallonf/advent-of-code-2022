mod draw_ctx;
mod draw_utils;
mod js;
mod test_algo;

use std::{cell::RefCell, rc::Rc};

use ggez::{
    self,
    conf::{WindowMode, WindowSetup},
    glam::Vec2,
    graphics::{self, DrawParam},
    ContextBuilder, GameError,
};
use js::draw_runtime::DrawRuntime;
use js::watcher::Watcher;
use tap::Pipe;

struct AppState {
    draw_runtime: DrawRuntime,
    watcher: Watcher,
}

impl ggez::event::EventHandler<GameError> for AppState {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), GameError> {
        if self.watcher.is_dirty() {
            self.watcher
                .stop_watching()
                .map_err(|err| GameError::CustomError(err.to_string()))?;
            let mut runtime_ref = &mut self.draw_runtime;
            *runtime_ref = runtime_ref.restart();
            self.watcher
                .start_watching(&mut runtime_ref)
                .map_err(|err| GameError::CustomError(err.to_string()))?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        let runtime = &mut self.draw_runtime;
        let mut canvas = graphics::Canvas::from_frame(ctx, draw_utils::WHITE);

        let draw_result = runtime.draw(ctx, &mut canvas);
        let text_to_draw = match draw_result {
            Ok(it) => it,
            Err(err) => format!("error calling draw(): {:?}", err),
        };
        // TODO: trim this - with backtraces, the text can get so long that it crashes ggez
        let mut text = graphics::Text::new(&text_to_draw);

        text.set_scale(16.0);
        canvas.draw(
            &mut text,
            DrawParam::default()
                .dest(Vec2::new(8.0, 8.0))
                .color(draw_utils::BLACK),
        );

        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut initial_state = AppState {
        watcher: Watcher::new()?,
        draw_runtime: DrawRuntime::new("./scripts/puzzles/test_algo/viz.js"),
    };

    initial_state
        .watcher
        .start_watching(&mut initial_state.draw_runtime)?;

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

    ggez::event::run(ctx, event_loop, initial_state);
}
