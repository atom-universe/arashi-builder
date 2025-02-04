use async_std::path::{Path, PathBuf};
use async_std::stream::StreamExt;
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
        specifier: Url::parse("file:///temporaryFile.ts").unwrap(),
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

/// 处理特殊标记的模块路径，找到其在 node_modules 中的具体位置
pub async fn resolve_module_path(root_dir: &str, module_path: &str) -> Option<PathBuf> {
    // 将模块路径拆分为包名和子路径，这是为了处理一种复杂的情况：
    // 比如，react-dom/client 在 pnpm 下的位置是：
    // node_modules/.pnpm/react-dom@xxx/node_modules/react-dom/client.js
    let parts: Vec<&str> = module_path.splitn(2, '/').collect();

    let (package_name, sub_path) = match parts.as_slice() {
        [pkg] => (*pkg, None),
        [pkg, path] => (*pkg, Some(*path)),
        _ => return None,
    };

    let pnpm_lock = Path::new(root_dir).join("pnpm-lock.yaml");
    let npm_lock = Path::new(root_dir).join("package-lock.json");

    let base_path = if async_std::fs::metadata(&pnpm_lock).await.is_ok() {
        // 首先尝试 .pnpm/node_modules 路径
        let pnpm_modules_path = Path::new(root_dir)
            .join("node_modules/.pnpm/node_modules")
            .join(package_name);

        if async_std::fs::metadata(&pnpm_modules_path).await.is_ok() {
            pnpm_modules_path
        } else {
            // 如果找不到，尝试在 .pnpm 目录下查找带版本号的包
            let pnpm_dir = Path::new(root_dir).join("node_modules/.pnpm");

            // 读取目录内容——版本号需要读取 pnpm-lock.yaml 等文件来确定
            if let Ok(mut entries) = async_std::fs::read_dir(&pnpm_dir).await {
                while let Some(entry) = entries.next().await {
                    if let Ok(entry) = entry {
                        let file_name = entry.file_name().into_string().unwrap_or_default();
                        if file_name.starts_with(&format!("{package_name}@")) {
                            let package_path = entry.path().join("node_modules").join(package_name);
                            if async_std::fs::metadata(&package_path).await.is_ok() {
                                return resolve_package_entry(&package_path, sub_path).await;
                            }
                        }
                    }
                }
            }
            pnpm_modules_path
        }
    } else if async_std::fs::metadata(&npm_lock).await.is_ok() {
        Path::new(root_dir).join("node_modules").join(package_name)
    } else {
        panic!("No lock file found. Please run `npm install` or `pnpm install` first.")
    };

    // （如果是形如 react-dom/client， 有二级路径，那么还需要进一步解析）
    // 找到具体的入口文件，比如什么 main.js index.js 之类的
    resolve_package_entry(&base_path, sub_path).await
}

async fn resolve_package_entry(package_path: &Path, sub_path: Option<&str>) -> Option<PathBuf> {
    if !async_std::fs::metadata(package_path).await.is_ok() {
        return None;
    }

    // 如果有子路径，直接尝试解析
    if let Some(sub) = sub_path {
        let full_path = package_path.join(sub);

        // 比如 react-dom/client,
        // 由于不知道 client.xx 到底是什么文件（还可能是文件夹），所以这里遍历下。。。
        let extensions = [".js", ".ts", "/index.js", "/index.ts"];
        for ext in extensions.iter() {
            let path_with_ext = if sub.ends_with(".js") {
                full_path.clone()
            } else {
                full_path.with_extension(ext.trim_start_matches('.'))
            };

            if async_std::fs::metadata(&path_with_ext).await.is_ok() {
                return Some(path_with_ext);
            }
        }
    }

    // 如果没有子路径或子路径不存在，尝试读取 package.json
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

    // 如果以上都没找到，那么就报错
    panic!(
        "No entry file found for package: {}",
        package_path.display()
    );
}

/// 将 CommonJS 模块转换为 ESM 格式
pub fn transform_cjs_to_esm(content: &str, module_name: &str) -> String {
    // 添加 CommonJS 环境的模拟实现
    let cjs_shim = r#"
const process = {
    env: {
        NODE_ENV: 'development'
    }
};
const exports = {};
const module = { exports };
"#;

    if content.contains("process.env.NODE_ENV") {
        // 提取 require 路径
        if let Some(dev_path) = content
            .lines()
            .find(|line| line.contains("development.js"))
            .and_then(|line| {
                line.split("require('")
                    .nth(1)
                    .map(|s| s.split("')").next().unwrap_or(""))
            })
        {
            let clean_path = dev_path.trim_start_matches("./");
            return format!(
                "{}\nexport default await import('/@modules/{}/{}')",
                cjs_shim,
                module_name.split('/').next().unwrap_or(module_name),
                clean_path
            );
        }
    }

    // 其他情况的基本转换
    let base_content = content
        .replace("'use strict';", "")
        .replace("module.exports =", "export default")
        // 处理相对路径的 require
        .replace(
            "require('./",
            &format!(
                "await import('/@modules/{}/",
                module_name.split('/').next().unwrap_or(module_name)
            ),
        )
        .replace(
            "require(\"./",
            &format!(
                "await import(\"/@modules/{}/",
                module_name.split('/').next().unwrap_or(module_name)
            ),
        )
        // 处理第三方模块的 require
        .replace("require(\"", "await import(\"/@modules/")
        .replace("require('", "await import('/@modules/");

    // 如果包含任何 CommonJS 相关的变量，添加 shim
    if content.contains("process.env")
        || content.contains("exports.")
        || content.contains("module.exports")
    {
        format!("{}{}", cjs_shim, base_content)
    } else {
        base_content
    }
}
