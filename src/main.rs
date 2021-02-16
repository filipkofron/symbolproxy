use ntex::web;
use ntex::http;
use std;
use chrono::Local;

extern crate chrono;

fn get_remote_name(req: &web::dev::WebRequest<web::DefaultError>) -> String {
    let conn_info = req.connection_info();
    return String::from(conn_info.remote().unwrap_or("Unknown"));
}

fn sanitize_path(root: &std::path::Path, path_req: &str) -> std::path::PathBuf {
    let no_back = std::path::PathBuf::from(path_req.strip_prefix("/").unwrap_or(path_req).replace("..", ""));
    let mut full_path = root.join(&no_back).canonicalize().unwrap_or(std::path::PathBuf::from(&root));
    if !full_path.starts_with(&root) {
        println!("{} doesn't start with {}", full_path.to_str().unwrap_or(""), root.to_str().unwrap_or(""));
        full_path = std::path::PathBuf::from(&root).join("Invalid path");
    }
    return full_path;
}

async fn symbol_service(req: web::dev::WebRequest<web::DefaultError>) -> Result<web::dev::WebResponse, web::Error> {
    let args: Vec<String> = std::env::args().collect();
    let store_path = String::from(&args[1]);
    let remote = get_remote_name(&req);

    let full_path = sanitize_path(&std::path::PathBuf::from(store_path).canonicalize().unwrap(), req.path());
    let full_path_str = full_path.to_str().unwrap_or("Invalid");

    let link = std::fs::read_to_string(&full_path);
    let link_str = String::from(link.as_deref().unwrap_or("INVALID"));
    println!("[{}][{}]: {} -> {}", Local::now().format("%Y-%m-%d %H:%M:%S"), remote, full_path_str, link_str);
    
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

    let checked_path = match std::path::PathBuf::from(&args[1]).canonicalize() {
        Ok(path) => path,
        Err(err) => panic!("Invalid path: {} error: {}", &args[1], err.to_string())
    };

    println!("Serving {} on {}:{}", checked_path.to_str().unwrap_or("INVALID"), &args[2], &args[3]);

    let app = || web::App::new().service(web::service("*").finish(symbol_service));
    web::server(app)
        .bind(format!("{}:{}", &args[2], &args[3]))?
        .run()
        .await
}
