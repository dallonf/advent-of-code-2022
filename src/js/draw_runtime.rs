use anyhow::{anyhow, Error, Result};
use deno_core::{
    v8::{self, Global},
    JsRuntime, ModuleLoader,
};
use std::{fmt::Display, rc::Rc, sync::Arc};
use tap::prelude::*;

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

pub type InitResult<T> = std::result::Result<T, InitError>;

pub struct DrawRuntime(InitResult<DrawRuntimeData>);

struct DrawRuntimeData {
    js_runtime: JsRuntime,
    draw_fn: v8::Global<v8::Function>,
}

impl DrawRuntime {
    pub fn new(module_path: &str) -> Self {
        let tk_runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(it) => it,
            Err(err) => return DrawRuntime(Err(anyhow!(err).into())),
        };

        let data = match tk_runtime.block_on(async {
            let viz_module_path = deno_core::resolve_path(module_path)?;
            let loader = Rc::new(deno_core::FsModuleLoader);
            let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
                module_loader: Some(loader.clone()),
                ..Default::default()
            });
            js_runtime
                .execute_script("[aoc2022:runtime.js]", include_str!("../runtime.js"))
                .unwrap();

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
                draw_fn,
            })
        }) {
            Ok(it) => it,
            Err(err) => {
                return DrawRuntime(Err(anyhow!(err).context("Setting up JS runtime").into()))
            }
        };

        DrawRuntime(Ok(data))
    }

    pub fn draw(&mut self) -> Result<String> {
        let DrawRuntimeData {
            js_runtime,
            draw_fn,
        } = match &mut self.0 {
            Ok(data) => data,
            Err(err) => return Err(anyhow!(err.clone().0)),
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
}
