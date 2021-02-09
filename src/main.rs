use ntex::web;
use ntex::http;
use std;

async fn my_service(req: web::dev::WebRequest<web::DefaultError>) -> Result<web::dev::WebResponse, web::Error> {
    let path = String::from(req.path());
    println!("Path: {}", path);
    Ok(req.into_response(
        http::Response::PermanentRedirect()
            .header("Location", String::from("http://PRG-WS-SROS0213") + path.as_str())
            .header(http::header::CONTENT_TYPE, "application/octet-stream")
            .finish()))
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    let my_lambda = || web::App::new().service(web::service("*").finish(my_service));
    web::server(my_lambda)
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
