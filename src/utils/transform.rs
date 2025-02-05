use async_std::path::{Path, PathBuf};
use async_std::stream::StreamExt;
use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_parser::{ParseOptions, Parser, ParserReturn};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{JsxOptions, JsxRuntime, TransformOptions, Transformer};
use regex::Regex;
use serde_json;

/// 转换 ts 和 tsx => js
pub fn transform_typescript(source: &str, is_tsx: bool) -> String {
    // 设置内存分配器
    let allocator = Allocator::default();
    let source_type = if is_tsx {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };

    // 解析源码
    let ParserReturn {
        mut program,
        errors,
        panicked,
        ..
    } = Parser::new(&allocator, source, source_type)
        .with_options(ParseOptions {
            parse_regular_expression: true,
            ..ParseOptions::default()
        })
        .parse();
    // println!("ast: \n{:?}\n", program);
    if panicked || !errors.is_empty() {
        return format!("Parse Error: {:?}", errors);
    }

    let ret = SemanticBuilder::new()
        // Estimate transformer will triple scopes, symbols, references
        .with_excess_capacity(2.0)
        .build(&program);
    if !ret.errors.is_empty() {
        println!("Semantic Errors:");
        for error in ret.errors {
            let error = error.with_source_code(source.to_string());
            println!("{error:?}");
        }
    }

    let (symbols, scopes) = ret.semantic.into_symbol_table_and_scope_tree();
    let transform_options = TransformOptions {
        jsx: JsxOptions {
            development: false,           // 使用生产模式
            runtime: JsxRuntime::Classic, // 使用经典模式，生成 React.createElement
            ..JsxOptions::default()
        },
        ..TransformOptions::enable_all()
    };

    let ret = Transformer::new(
        &allocator,
        std::path::Path::new("virtual.tsx"),
        &transform_options,
    )
    .build_with_symbols_and_scopes(symbols, scopes, &mut program);
    if !ret.errors.is_empty() {
        println!("Transformer Errors:");
        for error in ret.errors {
            let error = error.with_source_code(source.to_string());
            println!("{error:?}");
        }
    }
    CodeGenerator::new().build(&program).code
    // let result = CodeGenerator::new()
    //     .with_options(CodegenOptions {
    //         ..CodegenOptions::default()
    //     })
    //     .build(&program);
    // codegen.set_source_type(source_type); // 设置源码类型，让它知道要处理 JSX
    // let result = codegen.build();

    // result.code
}

// pub async fn transform_typescript(content: &str, is_tsx: bool) -> String {
//     let result = esbuild::transform(
//         content,
//         TransformOptions {
//             loader: if is_tsx { "tsx" } else { "ts" },
//             target: Some("es2020"),
//             format: Some(Format::Esm),
//             ..Default::default()
//         },
//     )?;
//     result.code
// }

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
