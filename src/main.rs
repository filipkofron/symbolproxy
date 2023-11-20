use ntex::web;
use ntex::http;
use sqlx::Pool;
use sqlx::Sqlite;
use std::str::FromStr as _;
use std::sync::Arc;
use std::sync::Mutex;
use std::{self, ops::Deref};
use std::time::Duration;
use chrono::Local;
use log::{info, warn, LevelFilter};
use log4rs::{append::console::ConsoleAppender, config::{Appender, Root}};
use log4rs::Config;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

fn get_remote_name(req: &ntex::web::WebRequest<web::DefaultError>) -> String {
    let conn_info = req.connection_info();
    return String::from(conn_info.remote().unwrap_or("Unknown"));
}

fn sanitize_path(root: &std::path::Path, path_req: &str) -> std::path::PathBuf {
    let no_back = std::path::PathBuf::from(path_req.strip_prefix("/").unwrap_or(path_req).replace("..", ""));
    let mut full_path = std::path::PathBuf::from(root);
    for part in no_back.iter() {
        full_path = full_path.join(part);
    }
    full_path = full_path.canonicalize().unwrap_or(std::path::PathBuf::from(&root));

    if !full_path.starts_with(&root) {
        println!("{} doesn't start with {}", full_path.to_str().unwrap_or(""), root.to_str().unwrap_or(""));
        full_path = std::path::PathBuf::from(&root).join("Invalid path");
    }
    return full_path;
}

async fn symbol_service(req : web::WebRequest<web::DefaultError>, shared_pool: Arc<Mutex<Pool<Sqlite>>>) -> Result<web::WebResponse, web::Error> {
    let args: Vec<String> = std::env::args().collect();
    let store_path = String::from(args[1].deref().replace("\\", "/"));
    let remote = get_remote_name(&req).replace("\\", "/");

	println!("Path request: {}", req.path());

    let full_path = sanitize_path(&std::path::PathBuf::from(store_path).canonicalize().unwrap(), req.path());
    let full_path_str = full_path.to_str().unwrap_or("Invalid");

    let pool = shared_pool.lock().unwrap();

    //let query = sqlx::query_as::<_, Symbol>("SELECT url FROM SymbolModel WHERE filename = ? AND hash = ?").bind("ReducerEngine.dll").bind("58FA40628C000");
    let query = sqlx::query_as::<_, Symbol>("SELECT url FROM SymbolModel WHERE filename = ? AND hash = ?").bind("ReducerEngine.dll").bind("58FA40628C000");
    let result = query.fetch_one(&pool.to_owned()).await;
    match result {
        Ok(result) => {
            println!("URL: {}", result.url)
        }
        Err(e) => {
            println!("Error: {}", e)
        }
    }

    let link = std::fs::read_to_string(&full_path);
    let link_str = String::from(link.as_deref().unwrap_or("INVALID"));
    println!(" |- [{}][{}]: {} -> {}", Local::now().format("%Y-%m-%d %H:%M:%S"), remote, full_path_str, link_str);
    
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

    let database_file = "/Volumes/SSD/Temp/db/db.sqlite";
    let database_url = format!("sqlite://{}", database_file);
    let pool_timeout = Duration::from_secs(30);
    // with pool_max_connections = 1, the pool timeout. maybe related to https://github.com/launchbadge/sqlx/issues/1210
    let pool_max_connections = 8;

    //let conn = SqliteConnectOptions::from_str("sqlite://data.db")?
    //     .journal_mode(SqliteJournalMode::Wal)
    //     .read_only(true)
    //     .connect().await?;
    let from_str = SqliteConnectOptions::from_str(&database_url);
    let connection_options = from_str.unwrap()
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

    // let connection = sqlite::open(":memory:").unwrap();

    // let query_symbol_model = "
    //     CREATE TABLE symbolmodel (hash TEXT, filename TEXT, url TEXT, store_path TEXT);
    //     INSERT INTO symbolmodel VALUES ('DEADBEEF', 'boo.exe', 'http://example.com/boo.exe', NULL);
    //     INSERT INTO symbolmodel VALUES ('C000FFEE', 'doo.dll', NULL, 'store/doo.dll');
    // ";
    // connection.execute(query_symbol_model).unwrap();

    // let query_source_model = "
    //     CREATE TABLE sourcemodel (path TEXT, loaded BOOLEAN, failure_count INTEGER);
    //     CREATE UNIQUE INDEX idx_sourcemodel_path 
    //     ON sourcemodel (path);
    //     INSERT INTO sourcemodel VALUES ('http://example.com/src/boo.exe', true, 0);
    //     INSERT INTO sourcemodel VALUES ('http://example.com/src/doo.dll', true, 1);
    // ";
    // connection.execute(query_source_model).unwrap();

    // let query_symbols = "SELECT * FROM symbolmodel";

    // connection
    //     .iterate(query_symbols, |pairs| {
    //         for &(name, value) in pairs.iter() {
    //             println!("{} = {}", name, value.unwrap_or("NULL"));
    //             println!("---------");
    //         }
    //         true
    //     })
    // .unwrap();

    // // https://docs.rs/sqlite/latest/sqlite/


    println!("Just crash now :).");
    println!(".");
    println!(".");



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

    info!("Serving {} on {}:{}", checked_path.to_str().unwrap_or("INVALID"), &args[2], &args[3]);

    //let app = || web::App::new().service(web::service("*").finish(symbol_service));

    let sqlite_pool_copy = sqlite_pool_arc.clone();
    
    let app = move ||
    {
        let sqlite_pool_copy2 = sqlite_pool_copy.clone();
        web::App::new()
            .service(web::service("*").finish(move |req: web::WebRequest<web::DefaultError>| {
                symbol_service(req, sqlite_pool_copy2.clone())
            }))
    };
    web::server(app)
        .bind(format!("{}:{}", &args[2], &args[3]))?
        .run()
        .await
}
