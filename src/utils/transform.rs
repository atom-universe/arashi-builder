use deno_ast::{MediaType, ParseParams, SourceTextInfo};
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
