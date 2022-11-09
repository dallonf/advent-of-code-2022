use crate::{
    draw_utils::{self, str_to_color},
    framework::Event,
    prelude::*,
};
use ggez::{
    glam::Vec2,
    graphics::{self, Canvas, Color, DrawParam, FillOptions},
};
use rlua::prelude::*;
use serde::Deserialize;
use std::{
    cell::RefCell,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use super::serialize::to_lua;

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
            Err(_) => Ok(vec![self.initial_module_path.clone()]),
        }
    }

    pub fn draw(&mut self, gfx_ctx: &ggez::Context, canvas: &mut Canvas) -> Result<String> {
        let DrawRuntimeData { lua, .. } = match &mut self.result {
            Ok(it) => it,
            Err(err) => return Err(anyhow!(err.0.clone())),
        };

        let text_result = lua.context(|ctx| {
            let canvas_cell = RefCell::new(canvas);
            ctx.scope(|scope| {
                let draw_ctx = ctx.create_table()?;
                draw_ctx.set(
                    "rectangle_fill",
                    scope.create_function_mut(
                        |_, (x, y, width, height, color): (f32, f32, f32, f32, String)| {
                            let color = str_to_color(&color).ok_or_else(|| {
                                LuaError::external(anyhow!("Invalid color: {color}"))
                            })?;
                            let shape = graphics::Mesh::new_rectangle(
                                gfx_ctx,
                                graphics::DrawMode::fill(),
                                graphics::Rect::new(0.0, 0.0, width, height),
                                color,
                            )
                            .map_err(|err| LuaError::external(err))?;
                            canvas_cell.borrow_mut().draw(&shape, Vec2::new(x, y));
                            Ok(())
                        },
                    )?,
                )?;
                draw_ctx.set(
                    "rectangle_outline",
                    scope.create_function_mut(
                        |_,
                         (x, y, width, height, color, line_width): (
                            f32,
                            f32,
                            f32,
                            f32,
                            String,
                            f32,
                        )| {
                            let color = str_to_color(&color).ok_or_else(|| {
                                LuaError::external(anyhow!("Invalid color: {color}"))
                            })?;
                            let shape = graphics::Mesh::new_rectangle(
                                gfx_ctx,
                                graphics::DrawMode::stroke(line_width),
                                graphics::Rect::new(0.0, 0.0, width, height),
                                color,
                            )
                            .map_err(|err| LuaError::external(err))?;
                            canvas_cell.borrow_mut().draw(&shape, Vec2::new(x, y));
                            Ok(())
                        },
                    )?,
                )?;
                #[derive(Debug)]
                enum VAlign {
                    Top,
                    Middle,
                    Bottom,
                }
                impl<'lua> FromLua<'lua> for VAlign {
                    fn from_lua(
                        lua_value: LuaValue<'lua>,
                        lua: LuaContext<'lua>,
                    ) -> LuaResult<Self> {
                        let as_str = String::from_lua(lua_value, lua)?;
                        match as_str.as_str() {
                            "top" => VAlign::Top,
                            "middle" => VAlign::Middle,
                            "bottom" => VAlign::Bottom,
                            other => {
                                return Err(LuaError::FromLuaConversionError {
                                    from: "string",
                                    to: "VAlign",
                                    message: Some(format!("Unexpected value: {other}")),
                                })
                            }
                        }
                        .pipe(Ok)
                    }
                }
                #[derive(Debug)]
                enum HAlign {
                    Left,
                    Middle,
                    Right,
                }
                impl<'lua> FromLua<'lua> for HAlign {
                    fn from_lua(
                        lua_value: LuaValue<'lua>,
                        lua: LuaContext<'lua>,
                    ) -> LuaResult<Self> {
                        let as_str = String::from_lua(lua_value, lua)?;
                        match as_str.as_str() {
                            "left" => HAlign::Left,
                            "middle" => HAlign::Middle,
                            "right" => HAlign::Right,
                            other => {
                                return Err(LuaError::FromLuaConversionError {
                                    from: "string",
                                    to: "HAlign",
                                    message: Some(format!("Unexpected value: {other}")),
                                })
                            }
                        }
                        .pipe(Ok)
                    }
                }
                #[derive(Debug, Default)]
                struct TextOpts {
                    size: Option<f32>,
                    v_align: Option<VAlign>,
                    h_align: Option<HAlign>,
                    color: Option<Color>,
                }
                impl<'lua> FromLua<'lua> for TextOpts {
                    fn from_lua(
                        lua_value: LuaValue<'lua>,
                        lua: LuaContext<'lua>,
                    ) -> LuaResult<Self> {
                        let as_table = LuaTable::from_lua(lua_value, lua)?;
                        TextOpts {
                            size: as_table.get("size")?,
                            v_align: as_table.get("v_align")?,
                            h_align: as_table.get("h_align")?,
                            color: as_table
                                .get::<_, Option<String>>("color")?
                                .and_then(|it| str_to_color(&it)),
                        }
                        .pipe(Ok)
                    }
                }
                draw_ctx.set(
                    "text",
                    scope.create_function_mut(
                        |ctx, (text, x, y, opts): (String, f32, f32, LuaValue)| {
                            let opts = Option::<TextOpts>::from_lua(opts, ctx)?.unwrap_or_default();
                            let mut text = graphics::Text::new(&text);
                            text.set_scale(opts.size.unwrap_or(16.0));
                            canvas_cell.borrow_mut().draw(
                                &mut text,
                                DrawParam::default()
                                    .dest(Vec2::new(x, y))
                                    .color(opts.color.unwrap_or(draw_utils::BLACK)),
                            );
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
                .get::<_, LuaValue>("ProcessEvent")?
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

    fn create_lua_draw_ctx<'lua: 'scope, 'scope>(
        ctx: LuaContext<'lua>,
        scope: &'scope mut LuaScope<'lua, 'scope>,
        gfx_ctx: &'scope ggez::Context,
        canvas: &'scope mut ggez::graphics::Canvas,
    ) -> Result<LuaTable<'lua>, Error> {
        let draw_ctx = ctx.create_table()?;
        draw_ctx.set(
            "fill_rectangle",
            scope.create_function_mut(|_, (x, y, width, height): (f32, f32, f32, f32)| {
                let shape = graphics::Mesh::new_rectangle(
                    gfx_ctx,
                    graphics::DrawMode::Fill(FillOptions::default()),
                    graphics::Rect::new(0.0, 0.0, width, height),
                    graphics::Color::BLACK,
                )
                .map_err(|err| LuaError::external(err))?;
                canvas.draw(&shape, Vec2::new(x, y));
                Ok(())
            })?,
        )?;

        Ok(draw_ctx)
    }
}
