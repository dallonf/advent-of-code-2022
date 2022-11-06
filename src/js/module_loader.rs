use std::cell::RefCell;

use deno_core::{url::Url, FsModuleLoader, ModuleLoader};

pub struct TrackingModuleLoader {
    loaded_files: RefCell<Vec<Url>>,
    backing: FsModuleLoader,
}

impl TrackingModuleLoader {
    pub fn new() -> Self {
        TrackingModuleLoader {
            loaded_files: RefCell::new(vec![]),
            backing: FsModuleLoader,
        }
    }

    pub fn loaded_files(&self) -> Vec<Url> {
        self.loaded_files.borrow().clone()
    }
}

impl ModuleLoader for TrackingModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        is_main: bool,
    ) -> Result<deno_core::ModuleSpecifier, anyhow::Error> {
        self.backing.resolve(specifier, referrer, is_main)
    }

    fn load(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
        maybe_referrer: Option<deno_core::ModuleSpecifier>,
        is_dyn_import: bool,
    ) -> std::pin::Pin<Box<deno_core::ModuleSourceFuture>> {
        {
            self.loaded_files
                .borrow_mut()
                .push(module_specifier.clone());
        }
        self.backing
            .load(module_specifier, maybe_referrer, is_dyn_import)
    }
}
