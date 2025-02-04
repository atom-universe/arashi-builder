use crate::utils::transform::is_js_or_ts_file;
use crate::utils::transform::process_imports;
use crate::utils::transform::resolve_module_path;
use tide::{Next, Request, Response, StatusCode};

#[derive(Debug, Clone)]
pub struct DependencyAnalysis {
    root_dir: String,
}

impl DependencyAnalysis {
    pub fn new(root_dir: String) -> Self {
        DependencyAnalysis { root_dir }
    }
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for DependencyAnalysis {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let path = req.url().path().to_string();

        if path.starts_with("/@modules/") {
            let module_path = path.trim_start_matches("/@modules/");
            if let Some(file_path) = resolve_module_path(&self.root_dir, module_path).await {
                match async_std::fs::read_to_string(&file_path).await {
                    Ok(content) => {
                        let mut res = Response::new(StatusCode::Ok);
                        res.set_content_type("application/javascript");
                        res.set_body(content);
                        return Ok(res);
                    }
                    Err(_) => return Ok(Response::new(StatusCode::InternalServerError)),
                }
            }
            return Ok(Response::new(StatusCode::NotFound));
        }

        let mut response = next.run(req).await;

        if is_js_or_ts_file(&path) {
            if let Some(body) = response.take_body().into_string().await.ok() {
                // 当一个文件中有 import，将其中第三方依赖的导入路径特殊标记一下
                // 后续真正请求这些第三方模块的时候，识别到这些标记，就走上面的代码逻辑
                let processed_content = process_imports(body).await;
                response.set_content_type("application/javascript");
                response.set_body(processed_content);
            }
        }

        Ok(response)
    }
}
