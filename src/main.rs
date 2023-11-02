#![feature(absolute_path)]

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::{
    http::StatusCode,
    web::{self},
    App, HttpRequest, HttpServer,
};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// port
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// host
    #[arg(long, default_value_t = String::from("0.0.0.0"))]
    host: String,

    /// file_or_dir
    #[clap(default_value_t = String::from("."))]
    file_or_dir: String,
}

#[derive(Debug, Clone)]
struct AppData {
    body: String,
    path: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let Cli {
        port,
        host,
        file_or_dir,
    } = cli.clone();
    let path = std::path::Path::new(&file_or_dir);

    let path = std::path::absolute(path).unwrap();

    let port = find_port::find_port("127.0.0.1", port).expect("");

    if !path.exists() {
        panic!("file_or_dir not found: {}", path.to_string_lossy());
    }

    let name = path
        .clone()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let body = format!(
        r"
    <html><head>
    <meta charset='utf-8'>
    <title>Index of /</title></head><body><h1>Index of /</h1><ul><li><a target='_blank' href='/static/{}'>{}</a></li></ul></body>
    </html>
    ",
        name, name
    );

    let app_data = AppData {
        body,
        path: path.to_string_lossy().to_string(),
    };

    let local_ip = local_ip_address::local_ip().unwrap();
    println!("rserve:");
    println!("http://{}:{}/", "localhost", port);
    println!("http://{}:{}/", local_ip, port);
    println!("http://{}:{}/", host, port);

    HttpServer::new(move || {
        let mut app = App::new().app_data(web::Data::new(app_data.clone())).wrap(
            Cors::default()
                .allow_any_header()
                .allow_any_method()
                .allow_any_origin()
                .max_age(3600),
        );

        if path.is_dir() {
            app = app.service(Files::new("/", path.clone()).show_files_listing())
        } else {
            app = app
                .service(web::resource("/").route(web::get().to(
                    |_req: HttpRequest, data: web::Data<AppData>| async move {
                        let s = data.body.clone();

                        let resp = actix_web::HttpResponse::new(StatusCode::from_u16(200).unwrap());
                        resp.set_body(s)
                    },
                )))
                .service(
                    web::resource(format!("/static/{}", name)).route(web::get().to(
                        |req: HttpRequest, data: web::Data<AppData>| async move {
                            let s = data.path.to_string();
                            let file = NamedFile::open(s).unwrap();
                            file.into_response(&req)
                        },
                    )),
                );
        };

        app
    })
    .bind((host, port))?
    .run()
    .await
}
