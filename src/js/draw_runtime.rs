use anyhow::{anyhow, Error, Result};
use deno_core::{
    error::AnyError,
    op,
    url::Url,
    v8::{self, Global},
    Extension, JsRuntime, ModuleLoader, OpState, Resource,
};
use ggez::{
    glam::Vec2,
    graphics::{self, Canvas, FillOptions},
};
use std::{cell::RefCell, fmt::Display, rc::Rc, sync::Arc};
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

pub enum DrawRuntimeInitResult {
    Succeeded(DrawRuntimeData),
    Error(InitError, Option<Url>),
}

impl DrawRuntimeInitResult {
    fn from_error(err: Error, url: Option<Url>) -> Self {
        DrawRuntimeInitResult::Error(err.into(), url)
    }
}

pub struct DrawRuntime {
    initial_module_path: String,
    result: DrawRuntimeInitResult,
}

pub struct DrawRuntimeData {
    js_runtime: JsRuntime,
    tracking_loader: Rc<TrackingModuleLoader>,
    draw_fn: v8::Global<v8::Function>,
    internal_utils: v8::Global<v8::Object>,
}

impl DrawRuntime {
    pub fn new(module_path: &str) -> Self {
        let tk_runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(it) => it,
            Err(err) => {
                return Self {
                    result: DrawRuntimeInitResult::from_error(anyhow!(err), None),
                    initial_module_path: module_path.to_string(),
                }
            }
        };

        let viz_module_path = match deno_core::resolve_path(module_path) {
            Ok(it) => it,
            Err(err) => {
                return Self {
                    result: DrawRuntimeInitResult::from_error(anyhow!(err), None),
                    initial_module_path: module_path.to_string(),
                }
            }
        };

        let data = match tk_runtime.block_on(async {
            let viz_module_path = deno_core::resolve_path(module_path)?;
            let loader = Rc::new(TrackingModuleLoader::new());

            let extension = Extension::builder()
                .ops(vec![
                    into_rust_obj::decl(),
                    op_unwrap_rust_pointer::decl(),
                    op_draw::decl(),
                ])
                .js(vec![("[aoc2022:runtime.js]", include_str!("./runtime.js"))])
                .build();

            let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
                module_loader: Some(loader.clone()),
                extensions: vec![extension],
                ..Default::default()
            });

            let internal_utils = {
                let result = js_runtime
                    .execute_script("[aoc2022:internal.js]", include_str!("./internal.js"))?;
                let mut scope = js_runtime.handle_scope();
                let as_obj = result.open(&mut scope).to_object(&mut scope).unwrap();
                Global::new(&mut scope, as_obj)
            };

            let source = loader.load(&viz_module_path, None, false).await?;
            let viz_module_id = js_runtime
                .load_main_module(
                    &viz_module_path,
                    Some(String::from_utf8_lossy(&source.code).to_string()),
                )
                .await?;

            let recv = js_runtime.mod_evaluate(viz_module_id);
            js_runtime.run_event_loop(false).await?;
            recv.await??;

            let viz_module = js_runtime.get_module_namespace(viz_module_id)?;
            let draw_fn = {
                let mut scope = js_runtime.handle_scope();
                let key = deno_core::v8::String::new(&mut scope, "draw").unwrap();
                let draw_fn = viz_module
                    .open(&mut scope)
                    .get(&mut scope, key.into())
                    .ok_or(anyhow!("No draw() function defined in viz.js"))?
                    .try_conv::<v8::Local<v8::Function>>()
                    .expect("draw is not a function");

                Global::new(&mut scope, draw_fn)
            };

            anyhow::Ok(DrawRuntimeData {
                js_runtime,
                tracking_loader: loader.clone(),
                draw_fn,
                internal_utils,
            })
        }) {
            Ok(it) => it,
            Err(err) => {
                return Self {
                    result: DrawRuntimeInitResult::from_error(
                        anyhow!(err).context("Setting up JS runtime"),
                        Some(viz_module_path),
                    ),
                    initial_module_path: module_path.to_string(),
                };
            }
        };

        DrawRuntime {
            result: DrawRuntimeInitResult::Succeeded(data),
            initial_module_path: module_path.to_string(),
        }
    }

    pub fn get_loaded_modules(&mut self) -> Result<Vec<Url>> {
        match &self.result {
            DrawRuntimeInitResult::Succeeded(DrawRuntimeData {
                tracking_loader, ..
            }) => Ok(tracking_loader.loaded_files()),
            DrawRuntimeInitResult::Error(_, Some(url)) => Ok(vec![url.clone()]),
            DrawRuntimeInitResult::Error(err, None) => Err(anyhow!(err.0.clone())),
        }
    }

    pub fn draw(&mut self, gfx_ctx: &ggez::Context, canvas: &mut Canvas) -> Result<String> {
        let DrawRuntimeData {
            js_runtime,
            draw_fn,
            internal_utils,
            ..
        } = match &mut self.result {
            DrawRuntimeInitResult::Succeeded(it) => it,
            DrawRuntimeInitResult::Error(err, _) => return Err(anyhow!(err.0.clone())),
        };

        // AAAAAAAH
        // we need to get a raw pointer to this context in order to
        // send it to V8
        // so we'll need to be VERY CERTAIN to get rid of these pointers before
        // ending draw()
        struct DrawContextResource {
            op_state: Rc<RefCell<OpState>>,
            resource_id: u32,
        }
        impl Drop for DrawContextResource {
            fn drop(&mut self) {
                self.op_state
                    .borrow_mut()
                    .resource_table
                    .close(self.resource_id)
                    .unwrap()
            }
        }
        impl DrawContextResource {
            fn new(op_state: Rc<RefCell<OpState>>, draw_ctx: DrawContext) -> Self {
                let resource_id = op_state.borrow_mut().resource_table.add(draw_ctx);
                Self {
                    op_state,
                    resource_id,
                }
            }
        }
        let draw_ctx = DrawContext { gfx_ctx, canvas };
        let draw_ctx_resource = DrawContextResource::new(js_runtime.op_state().clone(), draw_ctx);

        let mut scope = js_runtime.handle_scope();
        let js_draw_ctx = {
            let internal_utils = internal_utils.open(&mut scope);
            let undefined = v8::undefined(&mut scope);
            let key = v8::String::new(&mut scope, "createDrawCtx").unwrap().into();
            let create_draw_ctx = internal_utils
                .get(&mut scope, key)
                .unwrap()
                .try_conv::<v8::Local<v8::Function>>()
                .unwrap();
            let args =
                vec![v8::Number::new(&mut scope, draw_ctx_resource.resource_id as f64).into()];
            create_draw_ctx
                .call(&mut scope, undefined.into(), &args)
                .ok_or_else(|| anyhow!("Couldn't create a draw context"))?
        };
        let undefined = v8::undefined(&mut scope);
        let result = draw_fn
            .open(&mut scope)
            .call(&mut scope, undefined.into(), &vec![js_draw_ctx])
            .ok_or_else(|| anyhow!("Error calling draw()"))?
            .pipe(|it| {
                if it.is_string() {
                    Ok(it)
                } else {
                    Err(anyhow!(
                        "draw() should return a string, but it actually returned {}",
                        it.to_detail_string(&mut scope)
                            .ok_or(anyhow!("Bad value coming from draw()"))?
                            .to_rust_string_lossy(&mut scope)
                    ))
                }
            })?
            .to_rust_string_lossy(&mut scope);

        Ok(result)
    }

    pub fn restart(&self) -> Self {
        Self::new(&self.initial_module_path)
    }
}

struct PassToJs {
    x: i32,
}

impl PassToJs {
    fn print(&self) {
        println!("PassToJs: {}", self.x);
    }
}

impl Resource for PassToJs {}

#[op]
fn into_rust_obj(state: &mut OpState, x: i32) -> std::result::Result<u32, AnyError> {
    let resource_id = state.resource_table.add(PassToJs { x });
    Ok(resource_id)
}

#[op]
fn op_unwrap_rust_pointer(state: &mut OpState, pointer: u32) -> std::result::Result<i32, AnyError> {
    let obj = state.resource_table.get::<PassToJs>(pointer)?;
    obj.print();
    Ok(obj.x)
}

struct DrawContext {
    gfx_ctx: *const ggez::Context,
    canvas: *mut Canvas,
}
impl DrawContext {
    /// Must only be used in a synchronous op
    unsafe fn gfx_ctx(&self) -> &ggez::Context {
        std::mem::transmute(self.gfx_ctx)
    }

    /// Must only be used in a synchronous op
    unsafe fn canvas(&self) -> &mut Canvas {
        std::mem::transmute(self.canvas)
    }
}
impl Resource for DrawContext {}

#[op]
fn op_draw(
    state: &mut OpState,
    ctx_ptr: u32,
    shape: String,
    params: Vec<deno_core::serde_json::Value>,
) -> std::result::Result<(), AnyError> {
    let ctx = state.resource_table.get::<DrawContext>(ctx_ptr)?;

    match shape.as_str() {
        "rectangle" => {
            let mut params_iter = params.iter();
            let x = params_iter
                .next()
                .and_then(|it| it.as_f64())
                .ok_or_else(|| anyhow!("x must be number"))?;
            let y = params_iter
                .next()
                .and_then(|it| it.as_f64())
                .ok_or_else(|| anyhow!("y must be number"))?;
            let width = params_iter
                .next()
                .and_then(|it| it.as_f64())
                .ok_or_else(|| anyhow!("width must be number"))?;
            let height = params_iter
                .next()
                .and_then(|it| it.as_f64())
                .ok_or_else(|| anyhow!("height must be number"))?;
            let shape = graphics::Mesh::new_rectangle(
                unsafe { ctx.gfx_ctx() },
                graphics::DrawMode::Fill(FillOptions::default()),
                graphics::Rect::new(0.0, 0.0, width as f32, height as f32),
                graphics::Color::BLACK,
            )?;
            unsafe { ctx.canvas() }.draw(&shape, Vec2::new(x as f32, y as f32));
        }
        other => return Err(anyhow!("Unsupported shape: {other}")),
    }
    Ok(())
}
