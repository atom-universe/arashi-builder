use async_std::path::{Path, PathBuf};
use deno_ast::{MediaType, ParseParams, SourceTextInfo};
use regex::Regex;
use serde_json;
use url::Url;

pub fn transform_typescript(source: &str, is_tsx: bool) -> String {
    let media_type = if is_tsx {
        MediaType::Tsx
    } else {
        MediaType::TypeScript
    };

    let parse_params = ParseParams {
        // 这里需要一个虚拟的文件路径，否则会报错（实际上好像没啥影响）
        specifier: Url::parse("file:///dummy.ts").unwrap(),
        text_info: SourceTextInfo::new(source.into()),
        media_type,
        capture_tokens: true,
        scope_analysis: false,
        maybe_syntax: None,
    };

    match deno_ast::parse_module(parse_params) {
        Ok(parsed) => parsed.transpile(&Default::default()).unwrap().text,
        Err(e) => format!("Error: {:?}", e),
    }
}

pub fn is_js_or_ts_file(path: &str) -> bool {
    path.ends_with(".js")
        || path.ends_with(".jsx")
        || path.ends_with(".ts")
        || path.ends_with(".tsx")
}

pub async fn process_imports(content: String) -> String {
    let import_re = Regex::new(r#"(?m)^import\s+.*?from\s+["']([^"']+)["']"#).unwrap();
    let mut result = content.clone();

    for cap in import_re.captures_iter(&content) {
        let import_path = &cap[1];
        if !import_path.starts_with('.')
            && !import_path.starts_with('/')
            && !import_path.starts_with("http")
        {
            // 替换为 /@modules/ 开头的路径
            let new_path = format!("/@modules/{}", import_path);
            result = result.replace(&cap[0], &cap[0].replace(import_path, &new_path));
        }
    }

    result
}

pub async fn resolve_module_path(root_dir: &str, module_name: &str) -> Option<PathBuf> {
    let node_modules = Path::new(root_dir).join("node_modules").join(module_name);

    // pnpm 的路径结构
    let package_path = if node_modules.exists().await {
        node_modules
    } else {
        Path::new(root_dir)
            .join("node_modules/.pnpm")
            .join(module_name)
    };

    if async_std::fs::metadata(&package_path).await.is_ok() {
        let package_json = package_path.join("package.json");
        if let Ok(content) = async_std::fs::read_to_string(&package_json).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(entry) = json.get("module").or_else(|| json.get("main")) {
                    if let Some(entry_path) = entry.as_str() {
                        return Some(package_path.join(entry_path));
                    }
                }
            }
        }
        Some(package_path.join("index.js"))
    } else {
        None
    }
}
