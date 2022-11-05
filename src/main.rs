mod draw_ctx;
mod draw_utils;
mod test_algo;

use anyhow::anyhow;
use deno_core::{error::AnyError, op, serde_json, serde_v8, v8, Extension, ModuleLoader};
use draw_ctx::{DrawContext, DrawFn};
use ggez::{
    self,
    conf::{WindowMode, WindowSetup},
    graphics, ContextBuilder, GameError,
};
use lazy_static::__Deref;
use std::{
    borrow::Borrow,
    future::Future,
    rc::Rc,
    sync::{Mutex, TryLockError},
    thread,
};
use tap::prelude::*;

struct AppState {
    // current_draw_fn: DrawFn,
    // draw_ctx: &'static DrawContext,
}

impl ggez::event::EventHandler<GameError> for AppState {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        // match self.draw_ctx.draw_fn.try_lock() {
        //     Ok(mut draw_fn) => {
        //         let inner = draw_fn.take();
        //         if let Some(new_draw_fn) = inner {
        //             self.current_draw_fn = new_draw_fn;
        //         }
        //     }
        //     Err(TryLockError::WouldBlock) => {}
        //     Err(TryLockError::Poisoned(_)) => return Err(GameError::LockError),
        // }
        // let mut canvas = graphics::Canvas::from_frame(ctx, draw_utils::WHITE);
        // (self.current_draw_fn)(&mut canvas, ctx)?;
        // canvas.finish(ctx)?;
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

async fn run_js(file_path: &str) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path)?;
    let runjs_extension = Extension::builder()
        .ops(vec![
            // op_read_file::decl(),
            // op_write_file::decl(),
            // op_remove_file::decl(),
        ])
        .build();
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![runjs_extension],
        ..Default::default()
    });
    js_runtime
        .execute_script("[runjs:runtime.js]", include_str!("./runtime.js"))
        .unwrap();
    let mod_id = js_runtime.load_main_module(&main_module, None).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(false).await?;
    result.await?
}

fn main_js() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    if let Err(error) = runtime.block_on(run_js("./example.js")) {
        eprintln!("error: {}", error);
    }
}

fn main() -> anyhow::Result<()> {
    // let conf = ggez::conf::Conf::new();
    // let (ctx, event_loop) = ContextBuilder::new("aoc2022", "dallonf")
    //     .default_conf(conf)
    //     .window_mode(
    //         WindowMode::default()
    //             .resizable(false)
    //             .dimensions(1366.0, 768.0)
    //             .resize_on_scale_factor_change(false),
    //     )
    //     .window_setup(WindowSetup::default().title("Advent of Code 2022"))
    //     .build()
    //     .unwrap();

    // let draw_ctx: &'static DrawContext = Box::new(DrawContext {
    //     draw_fn: Mutex::new(None),
    // })
    // .pipe(|it| Box::leak(it));
    // thread::spawn(move || {
    //     test_algo::part_two_viz(draw_ctx).unwrap();
    // });

    let tk_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let async_setup = async {
        let viz_module = deno_core::resolve_path("./scripts/puzzles/test_algo/viz.js")?;
        let loader = Rc::new(deno_core::FsModuleLoader);
        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(loader.clone()),
            ..Default::default()
        });
        js_runtime
            .execute_script("[aoc2022:runtime.js]", include_str!("./runtime.js"))
            .unwrap();
        let source = loader.load(&viz_module, None, false).await?;
        let viz_module_id = js_runtime
            .load_main_module(
                &viz_module,
                Some(String::from_utf8_lossy(&source.code).to_string()),
            )
            .await?;

        let recv = js_runtime.mod_evaluate(viz_module_id);
        js_runtime.run_event_loop(false).await?;
        recv.await??;

        let viz_module = js_runtime.get_module_namespace(viz_module_id)?;
        let mut scope = js_runtime.handle_scope();
        let key = deno_core::v8::String::new(&mut scope, "draw").unwrap();
        let draw_fn = viz_module
            .open(&mut scope)
            .get(&mut scope, key.into())
            .ok_or(anyhow!("No draw() function defined in viz.js"))?
            .try_conv::<v8::Local<v8::Function>>()
            .expect("draw is not a function");
        let receiver = v8::undefined(&mut scope);
        let result = draw_fn.call(&mut scope, receiver.into(), &vec![]);
        let result_readable = serde_v8::from_v8::<serde_json::Value>(&mut scope, result.unwrap())?;
        dbg!(&result_readable);
        anyhow::Ok(())
    };

    tk_runtime.block_on(async_setup)?;

    // js_runtime.get_module_namespace(module_id)
    // js_runtime
    //     .execute_script("[runjs:runtime.js]", include_str!("./runtime.js"))
    //     .unwrap();

    // let initial_state = AppState {
    // current_draw_fn: Box::new(|_, _| Ok(())),
    // draw_ctx,
    // };
    // ggez::event::run(ctx, event_loop, initial_state);
    Ok(())
}
