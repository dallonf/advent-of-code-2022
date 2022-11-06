use anyhow::{anyhow, Error, Result};
use deno_core::{
    error::AnyError,
    op,
    url::Url,
    v8::{self, Global},
    Extension, JsRuntime, ModuleLoader, OpState, Resource,
};
use std::{fmt::Display, rc::Rc, sync::Arc};
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
                .ops(vec![into_rust_obj::decl(), op_unwrap_rust_pointer::decl()])
                .js(vec![(
                    "[aoc2022:runtime.js]",
                    include_str!("../runtime.js"),
                )])
                .build();

            let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
                module_loader: Some(loader.clone()),
                extensions: vec![extension],
                ..Default::default()
            });

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

    pub fn draw(&mut self) -> Result<String> {
        let DrawRuntimeData {
            js_runtime,
            draw_fn,
            ..
        } = match &mut self.result {
            DrawRuntimeInitResult::Succeeded(it) => it,
            DrawRuntimeInitResult::Error(err, _) => return Err(anyhow!(err.0.clone())),
        };

        let mut scope = js_runtime.handle_scope();
        let undefined = v8::undefined(&mut scope);
        let result = draw_fn
            .open(&mut scope)
            .call(&mut scope, undefined.into(), &vec![])
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
