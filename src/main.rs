use ntex::web;
use ntex::http;
use std;

async fn my_service(req: web::dev::WebRequest<web::DefaultError>) -> Result<web::dev::WebResponse, web::Error> {
    let path = String::from(req.path());
    println!("Path: {}", path);
    let link = std::fs::read_to_string(String::from("c:\\Data\\Projects\\symbolpublisher\\temp_store") + path.as_str());
    
    match &link {
        Ok(_) => Ok(req.into_response(
                    http::Response::PermanentRedirect()
                        .header("Location", link?)
                        .header(http::header::CONTENT_TYPE, "application/octet-stream")
                        .finish())),
        Err(_) => Ok(req.into_response(
                    http::Response::NotFound().finish())),
    }
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3
    {
        panic!("Invalid command line arguments!\n
        Path to the symbol store and port are required: symbolproxy.exe C:\\symbols 8080");
    }

    println!("Serving {} on port {}", &args[1], &args[2]);

    let my_lambda = || web::App::new().service(web::service("*").finish(my_service));
    web::server(my_lambda)
        .bind(format!("0.0.0.0:{}", &args[2]))?
        .run()
        .await
}
