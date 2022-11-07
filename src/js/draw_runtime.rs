use anyhow::{anyhow, Error, Result};
// use deno_core::{
//     error::AnyError,
//     op,
//     url::Url,
//     v8::{self, Global},
//     Extension, JsRuntime, ModuleLoader, OpState, Resource,
// };
use ggez::{
    glam::Vec2,
    graphics::{self, Canvas, FillOptions},
};
use rlua::prelude::*;
use std::{
    backtrace::{self, Backtrace},
    cell::RefCell,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};
use tap::prelude::*;

use super::module_loader::TrackingModuleLoader;

#[derive(Clone)]
pub struct InitError(Arc<Error>);

impl Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Debug for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InitError").field(&self.0).finish()
    }
}

impl std::error::Error for InitError {}

impl From<Error> for InitError {
    fn from(err: Error) -> Self {
        InitError(Arc::new(err))
    }
}

// pub enum DrawRuntimeInitResult {
//     Succeeded(DrawRuntimeData),
//     Error(InitError, Option<Url>),
// }

// impl DrawRuntimeInitResult {
//     fn from_error(err: Error, url: Option<Url>) -> Self {
//         DrawRuntimeInitResult::Error(err.into(), url)
//     }
// }

pub struct DrawRuntime {
    initial_module_path: PathBuf,
    result: Result<DrawRuntimeData, InitError>,
}

pub struct DrawRuntimeData {
    lua: Lua,
    // tracking_loader: Rc<TrackingModuleLoader>,
    // draw_fn: v8::Global<v8::Function>,
    // internal_utils: v8::Global<v8::Object>,
}

impl DrawRuntime {
    pub fn new(module_path: &Path) -> Self {
        let lua = Lua::new();
        let module_path = module_path.to_path_buf();
        match lua.context(|ctx| {
            println!("loading runtime...");

            println!("globals: {}", ctx.globals().len()?);

            let print: LuaFunction = ctx.globals().get("print")?;
            print.call::<_, ()>("hello from rust")?;

            ctx.load(include_str!("./runtime.lua"))
                .set_name("aoc2022:runtime.lua")?
                .exec()?;
            println!("loading source...");
            let source = fs::read(&module_path)?.pipe(|it| String::from_utf8(it))?;
            println!("{}", &source);
            println!("{}", module_path.to_str().unwrap().to_string());
            ctx.load(&source)
                .set_name(module_path.to_str().unwrap())?
                .exec()?;
            println!("loaded!");

            Ok(())
        }) {
            Ok(_) => (),
            Err(err) => {
                return DrawRuntime {
                    initial_module_path: module_path,
                    result: Err(InitError(Arc::new(err))),
                }
            }
        };

        DrawRuntime {
            result: Ok(DrawRuntimeData { lua }),
            initial_module_path: module_path,
        }
    }

    // pub fn get_loaded_modules(&self) -> Result<Vec<PathBuf>> {
    //     match &self.result {
    //         Ok(DrawRuntimeData { lua }) => {
    //             // lua.context(f)
    //             // let loaded_table
    //         },
    //         Err(err) => return Err(anyhow!(err.0.clone())),
    //     }
    // }

    pub fn draw(&mut self, gfx_ctx: &ggez::Context, canvas: &mut Canvas) -> Result<String> {
        let DrawRuntimeData { lua, .. } = match &mut self.result {
            Ok(it) => it,
            Err(err) => return Err(anyhow!(err.0.clone())),
        };

        let text_result = lua.context(|ctx| {
            ctx.scope(|scope| {
                let draw_ctx = ctx.create_table()?;
                draw_ctx.set(
                    "draw_rectangle",
                    scope.create_function_mut(
                        |_, (x, y, width, height): (f32, f32, f32, f32)| {
                            let shape = graphics::Mesh::new_rectangle(
                                gfx_ctx,
                                graphics::DrawMode::Fill(FillOptions::default()),
                                graphics::Rect::new(0.0, 0.0, width, height),
                                graphics::Color::BLACK,
                            )
                            .map_err(|err| LuaError::external(err))?;
                            canvas.draw(&shape, Vec2::new(x, y));
                            Ok(())
                        },
                    )?,
                )?;

                let draw_fn: LuaFunction = ctx.globals().get("Draw")?;
                let text_result: String = draw_fn.call(draw_ctx)?;
                anyhow::Ok(text_result)
            })
        })?;
        Ok(text_result)
    }

    pub fn restart(&self) -> Self {
        Self::new(&self.initial_module_path)
    }
}
