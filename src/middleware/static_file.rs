use crate::utils::fs;
use std::path::Path;
use tide::{Next, Request, Response, StatusCode};

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

        // println!("\nfile_path: {:?}\n", file_path);

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
