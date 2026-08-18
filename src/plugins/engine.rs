//! Rhai Scripting Engine container and sandboxed execution coordinator.

use crate::plugins::api::{
    self, HttpHandle, NoteHandle, PluginBuilder, StorageHandle, SystemHandle, ThemeHandle, UiHandle,
};
use rhai::{AST, Engine, Scope};

/// Maximum allowed evaluation steps per script execution to guarantee non-hanging behavior.
pub const MAX_SCRIPT_OPERATIONS: u64 = 500_000;
/// Maximum recursion call depth limit to prevent stack overflows.
pub const MAX_SCRIPT_CALL_LEVELS: usize = 50;
/// Maximum string allocation size (5MB).
pub const MAX_SCRIPT_STRING_SIZE: usize = 5_000_000;
/// Maximum array elements limit.
pub const MAX_SCRIPT_ARRAY_SIZE: usize = 100_000;
/// Maximum map keys limit.
pub const MAX_SCRIPT_MAP_SIZE: usize = 100_000;

/// Container wrapping the configured Rhai execution engine.
pub struct PluginEngine {
    engine: Engine,
}

impl Default for PluginEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginEngine {
    /// Creates and configures a new sandboxed Rhai engine instance with default resource limits.
    pub fn new() -> Self {
        Self::with_limits(
            MAX_SCRIPT_OPERATIONS,
            MAX_SCRIPT_CALL_LEVELS,
            MAX_SCRIPT_STRING_SIZE,
            MAX_SCRIPT_ARRAY_SIZE,
            MAX_SCRIPT_MAP_SIZE,
        )
    }

    /// Creates and configures a new sandboxed Rhai engine instance with custom resource quotas.
    pub fn with_limits(
        max_operations: u64,
        max_call_levels: usize,
        max_string_size: usize,
        max_array_size: usize,
        max_map_size: usize,
    ) -> Self {
        let mut engine = Engine::new();

        // Enforce strict security and resource limits
        engine.set_max_operations(max_operations);
        engine.set_max_call_levels(max_call_levels);
        engine.set_max_expr_depths(64, 64);
        engine.set_max_string_size(max_string_size);
        engine.set_max_array_size(max_array_size);
        engine.set_max_map_size(max_map_size);

        // Register Quicky Notes APIs
        api::register_apis(&mut engine);

        Self { engine }
    }

    /// Compiles Rhai script source code into an abstract syntax tree (AST).
    pub fn compile_script(&self, script: &str) -> Result<AST, String> {
        self.engine
            .compile(script)
            .map_err(|e| format!("Rhai compile error: {e}"))
    }

    /// Executes the `init(plugin)` registration hook on an AST.
    pub fn run_init(&self, ast: &AST, plugin_id: &str) -> Result<PluginBuilder, String> {
        let mut scope = Scope::new();
        let mut builder = PluginBuilder::new();
        builder.set_name(plugin_id.to_string());

        let has_init = ast
            .iter_functions()
            .any(|f| f.name == "init" && f.params.len() == 1);

        if has_init {
            scope.push("plugin", builder.clone());
            let _: rhai::Dynamic = self
                .engine
                .call_fn(&mut scope, ast, "init", (builder.clone(),))
                .map_err(|e| format!("Error in init() hook: {e}"))?;
        }

        Ok(builder)
    }

    /// Dispatches a hook function with standard `(note, ui, system, storage, http, theme)` context arguments and scope bindings.
    #[allow(clippy::too_many_arguments)]
    pub fn call_event_hook(
        &self,
        ast: &AST,
        function_name: &str,
        arg: Option<&str>,
        note_handle: &mut NoteHandle,
        ui_handle: &mut UiHandle,
        system_handle: &mut SystemHandle,
        storage_handle: &mut StorageHandle,
        http_handle: &mut HttpHandle,
        theme_handle: &mut ThemeHandle,
    ) -> Result<(), String> {
        let fn_match = ast.iter_functions().find(|f| f.name == function_name);

        let Some(fn_def) = fn_match else {
            return Ok(()); // Hook not implemented by this script, skip silently
        };

        let mut scope = Scope::new();
        scope.push("note", note_handle.clone());
        scope.push("ui", ui_handle.clone());
        scope.push("system", system_handle.clone());
        scope.push("storage", storage_handle.clone());
        scope.push("http", http_handle.clone());
        scope.push("theme", theme_handle.clone());

        match (fn_def.params.len(), arg) {
            (0, _) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(&mut scope, ast, function_name, ())
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (1, Some(arg_str)) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(&mut scope, ast, function_name, (arg_str.to_string(),))
                    .map_err(|e| format!("Error in {function_name}(\"{arg_str}\"): {e}"))?;
            }
            (1, None) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(&mut scope, ast, function_name, (note_handle.clone(),))
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (2, Some(arg_str)) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (arg_str.to_string(), note_handle.clone()),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (2, None) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (note_handle.clone(), ui_handle.clone()),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (3, Some(arg_str)) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (arg_str.to_string(), note_handle.clone(), ui_handle.clone()),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (3, None) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (
                            note_handle.clone(),
                            ui_handle.clone(),
                            system_handle.clone(),
                        ),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (4, Some(arg_str)) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (
                            arg_str.to_string(),
                            note_handle.clone(),
                            ui_handle.clone(),
                            system_handle.clone(),
                        ),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (4, None) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (
                            note_handle.clone(),
                            ui_handle.clone(),
                            system_handle.clone(),
                            storage_handle.clone(),
                        ),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (5, Some(arg_str)) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (
                            arg_str.to_string(),
                            note_handle.clone(),
                            ui_handle.clone(),
                            system_handle.clone(),
                            storage_handle.clone(),
                        ),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (5, None) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (
                            note_handle.clone(),
                            ui_handle.clone(),
                            system_handle.clone(),
                            storage_handle.clone(),
                            http_handle.clone(),
                        ),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (6, Some(arg_str)) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (
                            arg_str.to_string(),
                            note_handle.clone(),
                            ui_handle.clone(),
                            system_handle.clone(),
                            storage_handle.clone(),
                            http_handle.clone(),
                        ),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (6, None) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (
                            note_handle.clone(),
                            ui_handle.clone(),
                            system_handle.clone(),
                            storage_handle.clone(),
                            http_handle.clone(),
                            theme_handle.clone(),
                        ),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            (7, Some(arg_str)) => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(
                        &mut scope,
                        ast,
                        function_name,
                        (
                            arg_str.to_string(),
                            note_handle.clone(),
                            ui_handle.clone(),
                            system_handle.clone(),
                            storage_handle.clone(),
                            http_handle.clone(),
                            theme_handle.clone(),
                        ),
                    )
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
            _ => {
                let _: rhai::Dynamic = self
                    .engine
                    .call_fn(&mut scope, ast, function_name, ())
                    .map_err(|e| format!("Error in {function_name}(): {e}"))?;
            }
        }

        Ok(())
    }
}
