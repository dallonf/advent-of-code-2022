use crate::{framework::Event, prelude::*};
use ggez::{
    glam::Vec2,
    graphics::{self, Canvas, FillOptions},
};
use rlua::prelude::*;
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use super::serialize::{LuaSerializer, to_lua};

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

pub struct DrawRuntime {
    initial_module_path: PathBuf,
    result: Result<DrawRuntimeData, InitError>,
}

pub struct DrawRuntimeData {
    lua: Lua,
}

impl DrawRuntime {
    pub fn new(module_path: &Path) -> Self {
        let lua = Lua::new();
        let module_path = module_path.to_path_buf();
        match lua.context(|ctx| {
            ctx.load(include_str!("./runtime.lua"))
                .set_name("aoc2022:runtime.lua")?
                .exec()?;
            let source = fs::read(&module_path)?.pipe(|it| String::from_utf8(it))?;
            ctx.load(&source)
                .set_name(module_path.to_str().unwrap())?
                .exec()?;
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

    pub fn get_loaded_modules(&self) -> Result<Vec<PathBuf>> {
        match &self.result {
            Ok(DrawRuntimeData { lua }) => {
                let mut additional_packages = lua.context(|ctx| {
                    let package = ctx.globals().get::<_, LuaTable>("package")?;
                    let loaded = package
                        .get::<_, LuaTable>("loaded")?
                        .pairs::<String, LuaValue>()
                        .map(|pair| {
                            let package_name = pair.unwrap().0;

                            let actual_path: Option<String> = package
                                .get::<_, LuaFunction>("searchpath")
                                .unwrap()
                                .call::<_, LuaValue>((
                                    package_name,
                                    package.get::<_, LuaValue>("path").unwrap(),
                                ))
                                .unwrap()
                                .pipe(|it: LuaValue| {
                                    ctx.coerce_string(it).map_err(|err| anyhow!(err))
                                })
                                .unwrap()
                                .map(|it| it.to_str().map(|it| it.to_owned()))
                                .transpose()
                                .unwrap();
                            actual_path
                                .map(|it| PathBuf::from_str(&it))
                                .transpose()
                                .unwrap()
                        })
                        .collect::<Vec<_>>();

                    anyhow::Ok(loaded.into_iter().filter_map(|it| it).collect_vec())
                })?;

                additional_packages.push(self.initial_module_path.clone());
                Ok(additional_packages)
            }
            Err(err) => return Err(anyhow!(err.0.clone())),
        }
    }

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

    pub fn handle_event(&mut self, event: &Box<Event>) -> Result<()> {
        let DrawRuntimeData { lua, .. } = match &mut self.result {
            Ok(it) => it,
            Err(err) => return Err(anyhow!(err.0.clone())),
        };

        lua.context(|ctx| {
            let handle_fn: Option<LuaFunction> = ctx
                .globals()
                .get::<_, LuaValue>("HandleEvent")?
                .pipe(|it| {
                    if it.type_name() == "nil" {
                        None
                    } else {
                        Some(LuaFunction::from_lua(it, ctx))
                    }
                })
                .transpose()?;
            
            if let Some(handle_fn) = handle_fn {
                let lua_event = to_lua(ctx, event)?;
                handle_fn.call(lua_event)?;
            }

            anyhow::Ok(())
        })
    }

    pub fn restart(&self) -> Self {
        Self::new(&self.initial_module_path)
    }
}
