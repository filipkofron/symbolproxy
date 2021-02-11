use ntex::web;
use ntex::http;
use std;
use chrono::Local;

extern crate chrono;

fn get_remote_name(req: &web::dev::WebRequest<web::DefaultError>) -> String {
    let conn_info = req.connection_info();
    match conn_info.remote() {
        None => String::from("Unknown"),
        Some(name) => String::from(name),
    }
}

async fn symbol_service(req: web::dev::WebRequest<web::DefaultError>) -> Result<web::dev::WebResponse, web::Error> {
    let path = String::from(req.path());
    let args: Vec<String> = std::env::args().collect();
    let store_path = String::from(&args[1]);
    let remote = get_remote_name(&req);

    println!("[{}][{}]: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), remote, path);
    let link = std::fs::read_to_string(store_path + path.as_str());
    
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
    if args.len() != 4
    {
        panic!("Invalid command line arguments!\n
        Path to the symbol store, interface port are required: symbolproxy.exe C:\\symbols 0.0.0.0 8080");
    }

    println!("Serving {} on {}:{}", &args[1], &args[2], &args[3]);

    let app = || web::App::new().service(web::service("*").finish(symbol_service));
    web::server(app)
        .bind(format!("{}:{}", &args[2], &args[3]))?
        .run()
        .await
}
