use crate::utils::prebuild::DepCache;
use crate::utils::transform::{
    is_js_or_ts_file, process_imports, resolve_module_path, transform_cjs_to_esm,
};
use async_std::path::Path;
use std::sync::Arc;
use tide::{Next, Request, Response, StatusCode};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct DependencyAnalysis {
    root_dir: String,
    dep_cache: Arc<RwLock<DepCache>>,
}

impl DependencyAnalysis {
    pub async fn new(root_dir: String) -> Self {
        let dep_cache = Arc::new(RwLock::new(DepCache::new(Path::new(&root_dir)).await));
        Self {
            root_dir,
            dep_cache,
        }
    }
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for DependencyAnalysis {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let path = req.url().path().to_string();

        if path.starts_with("/@modules/") {
            let module_name = path.trim_start_matches("/@modules/");

            // 尝试获取或构建模块
            if let Some(pkg_path) = resolve_module_path(&self.root_dir, module_name).await {
                match self
                    .dep_cache
                    .write()
                    .await
                    .get_or_build(module_name, &pkg_path)
                    .await
                {
                    Ok(cached_path) => {
                        return Ok(Response::builder(200)
                            .content_type("application/javascript")
                            .body(async_std::fs::read_to_string(cached_path).await?)
                            .build());
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
