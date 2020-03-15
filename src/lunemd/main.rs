use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use liblunem::*;
use listenfd::ListenFd;
use config::*;
use notify::{RecommendedWatcher, DebouncedEvent, Watcher, RecursiveMode};
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::RwLock;
use std::time::Duration;
use std::path::Path;
use std::thread;
use directories::ProjectDirs;
use log::*;

#[macro_use]
extern crate lazy_static;


async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn api_status() -> impl Responder {
    HttpResponse::Ok().body("Getting status")
}

mod settings;

lazy_static! {
    pub static ref CONFIGFILE: RwLock<String> = RwLock::new(String::default());
    pub static ref CONFIGDATA: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let about = format!(
        "{}\nThis is the daemon applicaton which should be run at startup and in the backgroud",
        clap::crate_description!()
    );
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(&*about)
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

    let appname = clap::crate_name!();
    let mut default_config = String::from("/tmp");
    if let Some(project_dirs) = ProjectDirs::from("org", "darval", appname) {
        if let Some(config) = project_dirs.config_dir().to_str() {
            default_config = String::from(config);
        }
    }
    let config_dir = matches
        .value_of("config_dir")
        .unwrap_or_else(|| default_config.as_str());

    crate::CONFIGFILE.write().unwrap().push_str(format!("{}/lunemd.toml", config_dir).as_str());
    init_logging(&matches);
    thread::spawn(|| watch());
    info!("Reading config: {}", crate::CONFIGFILE.read().unwrap());
    settings::SETTINGS.write().unwrap().refresh().unwrap();

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/api/status", web::get().to(api_status))
        })
        .workers(4);
    
    let cd = crate::CONFIGDATA.read().unwrap().clone();
    let ip_port = format!("{}:{}", 
        cd["host"], 
        cd["port"]
    );

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind(ip_port)?
    };

    server.run().await
}

fn watch() {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    let configfile = &crate::CONFIGFILE.read().unwrap();
    let path = Path::new(configfile.as_str());
    watcher
        .watch(path, RecursiveMode::NonRecursive)
        .unwrap();

    // This is a simple loop, but you may want to use more complex logic here,
    // for example to handle I/O.
    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Write(_)) => {
                info!(" * lunemd.toml written; refreshing configuration ...");
                settings::SETTINGS.write().unwrap().refresh().unwrap();
                let hm = settings::SETTINGS.write().unwrap()
                    .clone().try_into::<HashMap<String, String>>().unwrap();
                *crate::CONFIGDATA.write().unwrap() = hm;
        
            }

            Err(e) => warn!("watch error: {:?}", e),

            _ => {
                // Ignore event
            }
        }
    }
}
