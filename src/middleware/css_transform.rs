use crate::utils::fs;
use std::path::Path;
use tide::{Next, Request, Response, StatusCode};

#[derive(Debug, Clone)]
pub struct CssTransform {
    pub root_dir: String,
}

impl CssTransform {
    pub fn new(root_dir: String) -> Self {
        CssTransform { root_dir }
    }
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for CssTransform {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let path: &str = req.url().path();
        // println!(
        //     "path: \n{}\n{}\n =====",
        //     path,
        //     req.header("Accept").clone().unwrap().to_string()
        // );
        if path.ends_with(".css") {
            // 如果发现这是一个 css 资源，
            // 那么就把它转化成 js 脚本，返回回去让前端动态执行
            let css_in_js = format!(
                r#"
                  const css = `{}`;
                  const style = document.createElement('style');
                  style.textContent = css;
                  document.head.appendChild(style);
                  export default css;
              "#,
                fs::read_file_content(Path::new(&self.root_dir).join(path.trim_start_matches('/')))
                    .unwrap()
            );

            let mut res = Response::new(StatusCode::Ok);
            res.set_content_type("application/javascript");
            res.set_body(css_in_js);
            Ok(res)
        } else {
            Ok(next.run(req).await)
        }
    }
}
