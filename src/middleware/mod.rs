use crate::utils::fs;
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;
use tide::{Next, Request, Response, StatusCode};

#[derive(Debug, Clone)]
pub struct Logger;

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for Logger {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        println!("测试");
        let response = next.run(req).await;
        Ok(response)
    }
}

#[derive(Debug, Clone)]
pub struct DependencyAnalysis {
    pub working_dir: String,
}

impl DependencyAnalysis {
    pub fn new(working_dir: String) -> Self {
        DependencyAnalysis { working_dir }
    }
}

pub struct Transform {
    pub file_path: String,
}

impl Transform {
    pub fn new(file_path: String) -> Self {
        Transform { file_path }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PnpmLock {
    packages: HashMap<String, PackageInfo>,
}

#[derive(Debug, serde::Deserialize)]
struct PackageInfo {
    resolution: Option<Resolution>,
    #[serde(default)]
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "peerDependencies")]
    peer_dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, serde::Deserialize)]
struct Resolution {
    integrity: String,
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for Transform {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let content = fs::read_file_content(&self.file_path).unwrap();
        Ok(Response::builder(200).body(content).build())
    }
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for DependencyAnalysis {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let mut pkg_name_2_pkg_path = HashMap::<String, String>::new();
        let path_string = std::path::Path::new(&self.working_dir)
            .join("pnpm-lock.yaml")
            .to_string_lossy()
            .to_string();

        let content = fs::read_file_content(&path_string).unwrap().to_string();
        let lock: PnpmLock = serde_yaml::from_str(&content).unwrap();

        for (package_name, _) in lock.packages {
            // println!("Package: {}", package_name);
            let module_name = package_name.split('@').collect::<Vec<&str>>()[0];
            println!("package_name/module_name: {}/{}", package_name, module_name);
            let subpath = format!(
                "node_modules/.pnpm/{}/node_modules/{}",
                package_name, module_name
            );

            let pkg_path = std::path::Path::new(&self.working_dir).join(subpath);
            pkg_name_2_pkg_path.insert(package_name, pkg_path.to_string_lossy().to_string());
        }

        let response = next.run(req).await;
        Ok(response)
    }
}

#[derive(Debug, Clone)]
pub struct StaticFiles {
    pub root_dir: String,
}

impl StaticFiles {
    pub fn new(root_dir: String) -> Self {
        StaticFiles { root_dir }
    }
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for StaticFiles {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let url_path = if req.url().path() == "/" {
            "index.html"
        } else {
            req.url().path().trim_start_matches('/')
        };

        let static_dirs = vec![&self.root_dir, "public"];
        let mut file_path = None;

        for dir in &static_dirs {
            let potential_path = Path::new(dir).join(url_path);
            if async_std::fs::metadata(&potential_path).await.is_ok() {
                file_path = Some(potential_path);
                break;
            }
        }

        if let Some(path) = file_path {
            let mime_type = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            let file = fs::read_file_bytes(&path).unwrap();
            let mut res = Response::new(StatusCode::Ok);
            res.set_content_type(mime_type.as_str());
            res.set_body(file);
            Ok(res)
        } else {
            Ok(Response::new(StatusCode::NotFound))
        }
    }
}
