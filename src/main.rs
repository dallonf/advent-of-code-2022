mod draw_ctx;
mod draw_utils;
mod js;
mod test_algo;

use anyhow::anyhow;
use deno_core::{error::AnyError, op, serde_json, serde_v8, v8, Extension, ModuleLoader};
use draw_ctx::{DrawContext, DrawFn};
use ggez::{
    self,
    conf::{WindowMode, WindowSetup},
    glam::Vec2,
    graphics::{self, DrawParam},
    ContextBuilder, GameError,
};
use js::draw_runtime::DrawRuntime;
use lazy_static::__Deref;
use notify::{RecursiveMode, Watcher};
use std::{
    borrow::Borrow,
    future::Future,
    rc::Rc,
    sync::{Arc, Mutex, TryLockError},
    thread,
};
use tap::prelude::*;

struct AppState {
    // current_draw_fn: DrawFn,
    // draw_ctx: &'static DrawContext,
    draw_runtime: Arc<Mutex<DrawRuntime>>,
}

impl ggez::event::EventHandler<GameError> for AppState {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        let mut runtime = self.draw_runtime.lock().unwrap();
        let draw_result = runtime.draw();
        let text_to_draw = match draw_result {
            Ok(it) => it,
            Err(err) => format!("error calling draw(): {:?}", err),
        };

        let mut canvas = graphics::Canvas::from_frame(ctx, draw_utils::WHITE);
        let mut text = graphics::Text::new(text_to_draw);
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

// #[op]
// async fn op_read_file(path: String) -> Result<String, AnyError> {
//     dbg!(&path);
//     let contents = tokio::fs::read_to_string(path).await?;
//     Ok(contents)
// }

// #[op]
// async fn op_write_file(path: String, contents: String) -> Result<(), AnyError> {
//     dbg!((&path, &contents));
//     tokio::fs::write(path, contents).await?;
//     Ok(())
// }

// #[op]
// fn op_remove_file(path: String) -> Result<(), AnyError> {
//     std::fs::remove_file(path)?;
//     Ok(())
// }

// async fn run_js(file_path: &str) -> Result<(), AnyError> {
//     let main_module = deno_core::resolve_path(file_path)?;
//     let runjs_extension = Extension::builder()
//         .ops(vec![
//             // op_read_file::decl(),
//             // op_write_file::decl(),
//             // op_remove_file::decl(),
//         ])
//         .build();
//     let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
//         module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
//         extensions: vec![runjs_extension],
//         ..Default::default()
//     });
//     js_runtime
//         .execute_script("[runjs:runtime.js]", include_str!("./runtime.js"))
//         .unwrap();
//     let mod_id = js_runtime.load_main_module(&main_module, None).await?;
//     let result = js_runtime.mod_evaluate(mod_id);
//     js_runtime.run_event_loop(false).await?;
//     result.await?
// }

fn main() -> anyhow::Result<()> {
    let mut runtime = DrawRuntime::new("./scripts/puzzles/test_algo/viz.js");
    let runtime = Arc::new(Mutex::new(runtime));

    let initial_state = AppState {
        draw_runtime: runtime.clone(),
        // current_draw_fn: Box::new(|_, _| Ok(())),
        // draw_ctx,
    };

    let mut watcher =
        notify::recommended_watcher(|res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => println!("Event {:?} for paths: {:?}", event.kind, event.paths),
            Err(err) => eprintln!("failed to watch files: {err}"),
        })?;

    let loaded_modules = runtime.lock().unwrap().get_loaded_modules()?;
    println!("watching files: {:?}", &loaded_modules);
    for module_url in loaded_modules.iter() {
        println!("setting up watch for {:?}", module_url);
        let path = module_url
            .to_file_path()
            .map_err(|_| anyhow!("Can't convert to path"))?;
        watcher.watch(path.as_path(), RecursiveMode::NonRecursive)?;
    }

    // let result = runtime.draw()?;
    // println!("draw() = {result}");

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

    // let draw_ctx: &'static DrawContext = Box::new(DrawContext {
    //     draw_fn: Mutex::new(None),
    // })
    // .pipe(|it| Box::leak(it));
    // thread::spawn(move || {
    //     test_algo::part_two_viz(draw_ctx).unwrap();
    // });

    ggez::event::run(ctx, event_loop, initial_state);
}
