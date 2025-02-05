use crate::utils::fs;
use crate::utils::transform::transform_typescript;
use std::path::Path;
use tide::{Next, Request, Response, StatusCode};

#[derive(Debug, Clone)]
pub struct TypescriptTransform {
    pub root_dir: String,
}

impl TypescriptTransform {
    pub fn new(root_dir: String) -> Self {
        TypescriptTransform { root_dir }
    }

    fn is_typescript_file(&self, path: &str) -> bool {
        path.ends_with(".ts") || path.ends_with(".tsx")
    }
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for TypescriptTransform {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let path = req.url().path();
        let is_tsx = path.ends_with(".tsx");

        if self.is_typescript_file(path) {
            let file_path: std::path::PathBuf =
                Path::new(&self.root_dir).join(path.trim_start_matches('/'));
            let content = fs::read_file_content(&file_path).unwrap();
            let transformed_content = transform_typescript(&content, is_tsx);
            println!("transformed_content: \n{}\n", transformed_content);
            let mut res = Response::new(StatusCode::Ok);
            res.set_content_type("application/javascript");
            res.set_body(transformed_content);
            Ok(res)
        } else {
            Ok(next.run(req).await)
        }
    }
}
