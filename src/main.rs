use ntex::web;
use ntex::http;
use sqlx::Pool;
use sqlx::Sqlite;
use std::str::FromStr as _;
use std::sync::Arc;
use std::sync::Mutex;
use std::{self};
use std::time::Duration;
use log::{info, warn, LevelFilter};
use log4rs::{append::console::ConsoleAppender, config::{Appender, Root}};
use log4rs::Config;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

fn get_remote_name(req: &ntex::web::WebRequest<web::DefaultError>) -> String {
    let conn_info = req.connection_info();
    return String::from(conn_info.remote().unwrap_or("Unknown"));
}

async fn symbol_service(req : web::WebRequest<web::DefaultError>, shared_pool: Arc<Mutex<Pool<Sqlite>>>) -> Result<web::WebResponse, web::Error> {
    let remote = get_remote_name(&req).replace("\\", "/");

    let path_parts: Vec<&str> = req.path().split('/').collect();

    let expected_size = 4;

    if !path_parts.len().eq(&expected_size) {
        warn!("Invalid request: {} from: {}", req.path(), remote);
        return Ok(req.into_response(http::Response::BadRequest().finish()));
    }

    let filename = path_parts[1];
    let hash = path_parts[2];
    let exact_filename = path_parts[3];

    if filename.is_empty() || hash.is_empty() || exact_filename.is_empty() || filename.ne(exact_filename) {
        warn!("Invalid request: {} from: {}", req.path(), remote);
        return Ok(req.into_response(http::Response::BadRequest().finish()));
    }

    let pool = shared_pool.lock().unwrap();

    let query = sqlx::query_as::<_, Symbol>("SELECT url FROM SymbolModel WHERE filename = ? AND hash = ?").bind(filename).bind(hash);
    let result = query.fetch_one(&pool.to_owned()).await;

    match &result {
        Ok(symbol) => {
            info!("Request: {} from: {} success: {}", req.path(), remote, symbol.url);
        }
        Err(_) => {
            warn!("Request: {} from: {} not found.", req.path(), remote);
        }
    }

    match result {
        Ok(symbol) => Ok(req.into_response(
                        http::Response::PermanentRedirect()
                            .header("Location", symbol.url)
                            .header(http::header::CONTENT_TYPE, "application/octet-stream")
                            .finish())),
        Err(_) => Ok(req.into_response(http::Response::NotFound().finish()))
    }
}

#[derive(sqlx::FromRow)]
struct Symbol {
    url: String
}

#[ntex::main]
async fn main() -> std::io::Result<()> {

    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4
    {
        panic!("Invalid command line arguments!\n
        Path to the symbol sqlite database, IP address to listen on, and interface port are both required.\n
        Example: symbolproxy.exe C:\\symbols\\db.sqlite 0.0.0.0 8080");
    }

    let checked_database_path = match std::path::PathBuf::from(&args[1]).canonicalize() {
        Ok(path) => path,
        Err(err) => panic!("Invalid path: {} error: {}", &args[1], err.to_string())
    };

    let database_url = format!("sqlite://{}", checked_database_path.to_str().unwrap());
    let pool_timeout = Duration::from_secs(30);
    let pool_max_connections = 128;

    let connection_options = SqliteConnectOptions::from_str(&database_url).unwrap()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(pool_timeout);

    let sqlite_pool = SqlitePoolOptions::new()
        .max_connections(pool_max_connections)
        .acquire_timeout(pool_timeout)
        .connect_with(connection_options)
        .await.unwrap();

    let sqlite_pool_arc = Arc::new(Mutex::new(sqlite_pool));

    info!("Serving {} on {}:{}", checked_database_path.to_str().unwrap_or("INVALID"), &args[2], &args[3]);
    
    let app = move ||
    {
        let sqlite_pool_captured = sqlite_pool_arc.clone();
        web::App::new()
            .service(web::service("*").finish(move |req: web::WebRequest<web::DefaultError>| {
                symbol_service(req, sqlite_pool_captured.clone())
            }))
    };
    web::server(app)
        .bind(format!("{}:{}", &args[2], &args[3]))?
        .run()
        .await
}
