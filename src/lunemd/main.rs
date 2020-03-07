use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use listenfd::ListenFd;
use liblunem::*;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn api_status() -> impl Responder {
    HttpResponse::Ok().body("Getting status")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let matches = clap::App::new(clap::crate_name!())
    .version(clap::crate_version!())
    .author(clap::crate_authors!())
    .about(clap::crate_description!())
    .arg(
        clap::Arg::with_name("config_dir")
            .short("c")
            .long("config_dir")
            .value_name("DIR")
            .help("Sets a custom config directory"),
    )
    .arg(
        clap::Arg::with_name("log_level")
            .short("l")
            .long("log_level")
            .value_name("debug|info|warn|error")
            .help("Sets the log level (default info) for the viewerator.log in the config directory"),
    )
    .arg(
        clap::Arg::with_name("host")
            .short("h")
            .long("host")
            .value_name("url")
            .help("Read JSON from different url rather than http://localhost"),
    )
    .arg(
        clap::Arg::with_name("input_file")
            .short("f")
            .long("input_file")
            .value_name("FILE")
            .help("Read JSON from file rather than http://localhost/api/status"),
    )
    .arg(
        clap::Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Runs in debug mode, which allows normal panics with trace"),
    )
    .get_matches();
    init_logging(&matches);

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| App::new()
        .route("/", web::get().to(index))
        .route("/api/status", web::get().to(api_status)));


    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:3000")?
    };

    server.run().await

}
