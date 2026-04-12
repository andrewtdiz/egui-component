use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{anyhow, Context as AnyhowContext};
use deno_ast::{
    DecoratorsTranspileOption, EmitOptions, ImportsNotUsedAsValues, JsxAutomaticOptions,
    JsxRuntime, MediaType, ParseParams, SourceMapOption, TranspileModuleOptions, TranspileOptions,
};
use deno_core::{
    extension, op2, resolve_import, JsRuntime, ModuleLoadOptions, ModuleLoadReferrer,
    ModuleLoadResponse, ModuleLoader, ModuleSource, ModuleSourceCode, ModuleSpecifier, ModuleType,
    OpState, ResolutionKind, RuntimeOptions as DenoRuntimeOptions,
};
use deno_error::JsErrorBox;

use crate::diagnostics::{push_log, RuntimeLogBuffer};

type SourceMapStore = Rc<RefCell<HashMap<String, Vec<u8>>>>;

#[derive(Debug, Clone)]
pub struct JsxRuntimeOptions {
    pub jsx_import_source: String,
    pub virtual_modules: Vec<VirtualModule>,
}

impl JsxRuntimeOptions {
    pub fn new(jsx_import_source: impl Into<String>) -> Self {
        Self {
            jsx_import_source: jsx_import_source.into(),
            virtual_modules: Vec::new(),
        }
    }

    pub fn with_virtual_module(
        mut self,
        import_specifier: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        self.virtual_modules
            .push(VirtualModule::new(import_specifier, source));
        self
    }
}

#[derive(Debug, Clone)]
pub struct VirtualModule {
    pub import_specifier: String,
    pub source: String,
}

impl VirtualModule {
    pub fn new(import_specifier: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            import_specifier: import_specifier.into(),
            source: source.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct RuntimeUpdate {
    pub commit_batches_json: Vec<String>,
    pub logs: RuntimeLogBuffer,
}

#[derive(Debug, Default)]
struct RuntimeState {
    commit_batches_json: Vec<String>,
    logs: RuntimeLogBuffer,
}

#[op2(fast)]
fn op_commit_mutations(
    state: &mut OpState,
    #[string] mutations_json: String,
) -> Result<(), JsErrorBox> {
    state
        .borrow_mut::<RuntimeState>()
        .commit_batches_json
        .push(mutations_json);
    Ok(())
}

#[op2(fast)]
fn op_host_log(
    state: &mut OpState,
    #[string] level: String,
    #[string] message: String,
) -> Result<(), JsErrorBox> {
    let runtime_state = state.borrow_mut::<RuntimeState>();
    push_log(&mut runtime_state.logs, format!("{level}: {message}"));
    Ok(())
}

extension!(
    clay_jsx_host,
    ops = [op_commit_mutations, op_host_log],
    docs = "Ops used by clay JSX host runtimes."
);

#[derive(Debug)]
struct JsxModuleLoader {
    import_aliases: HashMap<String, String>,
    virtual_sources: HashMap<String, String>,
    jsx_import_source: String,
    source_maps: SourceMapStore,
}

impl JsxModuleLoader {
    fn new(options: JsxRuntimeOptions) -> anyhow::Result<Self> {
        if options.jsx_import_source.trim().is_empty() {
            return Err(anyhow!("JSX import source must not be empty"));
        }

        let mut import_aliases = HashMap::new();
        let mut virtual_sources = HashMap::new();
        for (index, module) in options.virtual_modules.into_iter().enumerate() {
            if module.import_specifier.trim().is_empty() {
                return Err(anyhow!("virtual module import specifier must not be empty"));
            }
            let virtual_specifier = format!("clay-jsx-runtime:///virtual/{index}");
            import_aliases.insert(module.import_specifier, virtual_specifier.clone());
            virtual_sources.insert(virtual_specifier, module.source);
        }

        Ok(Self {
            import_aliases,
            virtual_sources,
            jsx_import_source: options.jsx_import_source,
            source_maps: Rc::new(RefCell::new(HashMap::new())),
        })
    }
}

impl ModuleLoader for JsxModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, deno_core::error::ModuleLoaderError> {
        if let Some(virtual_specifier) = self.import_aliases.get(specifier) {
            return ModuleSpecifier::parse(virtual_specifier).map_err(JsErrorBox::from_err);
        }

        resolve_import(specifier, referrer).map_err(JsErrorBox::from_err)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        ModuleLoadResponse::Sync(load_module(
            Rc::clone(&self.source_maps),
            &self.virtual_sources,
            &self.jsx_import_source,
            module_specifier,
        ))
    }

    fn get_source_map(&self, specifier: &str) -> Option<Cow<'_, [u8]>> {
        self.source_maps
            .borrow()
            .get(specifier)
            .map(|source_map| source_map.clone().into())
    }
}

pub struct RuntimeSession {
    entry_path: Option<PathBuf>,
    js_runtime: JsRuntime,
    tokio_runtime: tokio::runtime::Runtime,
}

impl std::fmt::Debug for RuntimeSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeSession")
            .field("entry_path", &self.entry_path)
            .finish_non_exhaustive()
    }
}

impl RuntimeSession {
    pub fn new(options: JsxRuntimeOptions) -> anyhow::Result<Self> {
        let js_runtime = JsRuntime::new(DenoRuntimeOptions {
            module_loader: Some(Rc::new(JsxModuleLoader::new(options)?)),
            extensions: vec![clay_jsx_host::init()],
            ..Default::default()
        });
        js_runtime
            .op_state()
            .borrow_mut()
            .put(RuntimeState::default());

        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to build Tokio runtime for deno_core")?;

        Ok(Self {
            entry_path: None,
            js_runtime,
            tokio_runtime,
        })
    }

    pub fn load(
        entry_path: &Path,
        options: JsxRuntimeOptions,
    ) -> anyhow::Result<(Self, RuntimeUpdate)> {
        let mut session = Self::new(options)?;
        session.load_main_module(entry_path)?;
        let update = session.take_update();
        Ok((session, update))
    }

    pub fn load_main_module(&mut self, entry_path: &Path) -> anyhow::Result<()> {
        let entry_path = absolute_path(entry_path)?;
        let main_module = ModuleSpecifier::from_file_path(&entry_path).map_err(|_| {
            anyhow!(
                "entry path is not a valid file URL: {}",
                entry_path.display()
            )
        })?;

        self.evaluate_main_module(&main_module)?;
        self.entry_path = Some(entry_path);
        Ok(())
    }

    pub fn execute_script(
        &mut self,
        name: &'static str,
        source: impl Into<String>,
    ) -> anyhow::Result<()> {
        self.js_runtime
            .execute_script(name, source.into())
            .map_err(|error| anyhow!("failed to execute script {name}: {error}"))?;
        self.run_event_loop()?;
        Ok(())
    }

    pub fn take_update(&mut self) -> RuntimeUpdate {
        let op_state = self.js_runtime.op_state();
        let mut op_state = op_state.borrow_mut();
        let state = op_state.borrow_mut::<RuntimeState>();
        RuntimeUpdate {
            commit_batches_json: std::mem::take(&mut state.commit_batches_json),
            logs: std::mem::take(&mut state.logs),
        }
    }

    pub fn entry_path(&self) -> Option<&Path> {
        self.entry_path.as_deref()
    }

    fn evaluate_main_module(&mut self, main_module: &ModuleSpecifier) -> anyhow::Result<()> {
        self.tokio_runtime.block_on(async {
            let module_id = self.js_runtime.load_main_es_module(main_module).await?;
            let result = self.js_runtime.mod_evaluate(module_id);
            self.js_runtime.run_event_loop(Default::default()).await?;
            result.await
        })?;
        Ok(())
    }

    fn run_event_loop(&mut self) -> anyhow::Result<()> {
        self.tokio_runtime
            .block_on(self.js_runtime.run_event_loop(Default::default()))?;
        Ok(())
    }
}

fn load_module(
    source_maps: SourceMapStore,
    virtual_sources: &HashMap<String, String>,
    jsx_import_source: &str,
    module_specifier: &ModuleSpecifier,
) -> Result<ModuleSource, deno_core::error::ModuleLoaderError> {
    if let Some(source) = virtual_sources.get(module_specifier.as_str()) {
        return Ok(module_source(module_specifier, source.clone()));
    }

    let path = module_specifier.to_file_path().map_err(|_| {
        JsErrorBox::generic("Only file:// and configured virtual modules are supported.")
    })?;
    let media_type = MediaType::from_path(&path);
    let (module_type, should_transpile) = match media_type {
        MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => (ModuleType::JavaScript, false),
        MediaType::Jsx
        | MediaType::TypeScript
        | MediaType::Mts
        | MediaType::Cts
        | MediaType::Dts
        | MediaType::Dmts
        | MediaType::Dcts
        | MediaType::Tsx => (ModuleType::JavaScript, true),
        MediaType::Json => (ModuleType::Json, false),
        _ => {
            return Err(JsErrorBox::generic(format!(
                "Unsupported module extension for {}",
                path.display()
            )));
        }
    };

    let code = std::fs::read_to_string(&path).map_err(JsErrorBox::from_err)?;
    let code = if should_transpile {
        let transpiled = transpile_module(module_specifier, media_type, code, jsx_import_source)
            .map_err(|error| JsErrorBox::generic(error.to_string()))?;
        if let Some(source_map) = transpiled.source_map {
            source_maps
                .borrow_mut()
                .insert(module_specifier.to_string(), source_map);
        }
        transpiled.code
    } else {
        code
    };

    Ok(ModuleSource::new(
        module_type,
        ModuleSourceCode::String(code.into()),
        module_specifier,
        None,
    ))
}

struct TranspiledModule {
    code: String,
    source_map: Option<Vec<u8>>,
}

fn transpile_module(
    module_specifier: &ModuleSpecifier,
    media_type: MediaType,
    code: String,
    jsx_import_source: &str,
) -> anyhow::Result<TranspiledModule> {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: module_specifier.clone(),
        text: code.into(),
        media_type,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })?;

    let emitted = parsed
        .transpile(
            &TranspileOptions {
                decorators: DecoratorsTranspileOption::Ecma,
                imports_not_used_as_values: ImportsNotUsedAsValues::Remove,
                jsx: Some(JsxRuntime::Automatic(JsxAutomaticOptions {
                    import_source: Some(jsx_import_source.to_owned()),
                    development: false,
                })),
                ..Default::default()
            },
            &TranspileModuleOptions::default(),
            &EmitOptions {
                source_map: SourceMapOption::Separate,
                inline_sources: true,
                remove_comments: true,
                ..Default::default()
            },
        )?
        .into_source();

    Ok(TranspiledModule {
        code: emitted.text,
        source_map: emitted.source_map.map(|source_map| source_map.into_bytes()),
    })
}

fn module_source(module_specifier: &ModuleSpecifier, source: String) -> ModuleSource {
    ModuleSource::new(
        ModuleType::JavaScript,
        ModuleSourceCode::String(source.into()),
        module_specifier,
        None,
    )
}

fn absolute_path(path: &Path) -> anyhow::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()
            .context("failed to resolve current directory")?
            .join(path))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{JsxRuntimeOptions, RuntimeSession};

    #[test]
    fn tsx_entrypoint_uses_configured_virtual_modules() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "clay";

render(<box answer={42} />);
"#,
        )
        .expect("tsx file should be written");

        let (_session, update) = RuntimeSession::load(&entry_path, test_options())
            .expect("tsx should transpile and render");

        assert_eq!(update.commit_batches_json.len(), 1);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&update.commit_batches_json[0])
                .expect("commit should be json"),
            serde_json::json!({ "type": "box", "props": { "answer": 42 } })
        );
    }

    fn test_options() -> JsxRuntimeOptions {
        JsxRuntimeOptions::new("clay")
            .with_virtual_module(
                "clay",
                r#"
export function render(element) {
  Deno.core.ops.op_commit_mutations(JSON.stringify(element));
}
"#,
            )
            .with_virtual_module(
                "clay/jsx-runtime",
                r#"
export const Fragment = Symbol.for("test.fragment");
export function jsx(type, props) {
  return { type, props: props ?? {} };
}
export const jsxs = jsx;
export const jsxDEV = jsx;
"#,
            )
    }
}
