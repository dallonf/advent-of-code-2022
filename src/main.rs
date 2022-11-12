mod draw_ctx;
mod draw_utils;
mod framework;
mod lua;
mod prelude;
mod test_algo;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use framework::{AsyncReportProgress, EventBus};
use ggez::{
    self,
    conf::{WindowMode, WindowSetup},
    glam::Vec2,
    graphics::{self, Color, DrawParam, Rect},
    ContextBuilder, GameError,
};
use lua::draw_runtime::DrawRuntime;
use lua::watcher::Watcher;
use prelude::*;

struct AppState {
    draw_runtime: DrawRuntime,
    watcher: Watcher,
    events: EventBus,
    events_processed: usize,
    processing_error: Option<Error>,
}

impl ggez::event::EventHandler<GameError> for AppState {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), GameError> {
        if self.watcher.is_dirty() {
            println!("Reloading Lua...");
            self.watcher
                .stop_watching()
                .map_err(|err| GameError::CustomError(err.to_string()))?;
            let mut runtime_ref = &mut self.draw_runtime;
            *runtime_ref = runtime_ref.restart();
            self.watcher
                .start_watching(&mut runtime_ref)
                .map_err(|err| GameError::CustomError(err.to_string()))?;

            self.events_processed = 0;
            self.processing_error = None;

            let events = self.events.lock().unwrap();
            if events.len() > 0 {
                println!("Replaying progress events...");
                for event in events.iter() {
                    if let Err(err) = self.draw_runtime.handle_event(event) {
                        self.processing_error = Some(err);
                        break;
                    }
                    self.events_processed += 1;
                }
                println!("Progress events done!");
            }

            println!("Reloaded!");
        } else if self.processing_error.is_none() {
            let events = self.events.lock().unwrap();
            if events.len() > self.events_processed {
                let unprocessed_events = events.iter().skip(self.events_processed);
                for event in unprocessed_events {
                    if let Err(err) = self.draw_runtime.handle_event(event) {
                        self.processing_error = Some(err);
                        break;
                    }
                    self.events_processed += 1;
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        let runtime = &mut self.draw_runtime;
        let mut canvas = graphics::Canvas::from_frame(ctx, draw_utils::WHITE);

        let draw_result = runtime.draw(ctx, &mut canvas);
        let draw_error = draw_result.err();
        let error_text = self
            .processing_error
            .as_ref()
            .map(|it| "Error while processing event:\n".to_string() + &it.to_string())
            .or(draw_error.map(|it| "Error in Draw():\n".to_string() + &it.to_string()));

        if let Some(error_text) = error_text {
            // TODO: trim this - with backtraces, the text can get so long that it crashes ggez
            let size = ctx.gfx.drawable_size();
            canvas.draw(
                &graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    Rect::new(0.0, 0.0, size.0, size.1),
                    Color::from_rgba(255, 255, 255, 200),
                )?,
                DrawParam::default(),
            );
            let mut text = graphics::Text::new(error_text);
            text.set_scale(16.0);
            canvas.draw(
                &mut text,
                DrawParam::default()
                    .dest(Vec2::new(8.0, 8.0))
                    .color(draw_utils::RED.to_owned()),
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let events = Arc::new(Mutex::new(vec![]));
    let mut initial_state = AppState {
        draw_runtime: DrawRuntime::new(&PathBuf::from("./scripts/puzzles/test_algo/viz.lua")),
        watcher: Watcher::new()?,
        events: events.clone(),
        events_processed: 0,
        processing_error: None,
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

    let thread_events = events.clone();
    thread::spawn(move || {
        test_algo::part_two(&AsyncReportProgress {
            event_bus: thread_events,
        });
    });

    ggez::event::run(ctx, event_loop, initial_state);
}
