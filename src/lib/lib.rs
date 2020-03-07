use directories::ProjectDirs;
use log::*;
use simplelog::*;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;

pub fn init_logging<'a>(matches: &clap::ArgMatches<'a>) {
    let appname = clap::crate_name!();
    let version = clap::crate_version!();
    let mut default_config = String::from("/tmp");
    if let Some(project_dirs) = ProjectDirs::from("org", "darval", appname) {
        if let Some(config) = project_dirs.config_dir().to_str() {
            default_config = String::from(config);
        }
    }
    let mut created_dir = false;
    let config_dir = matches
        .value_of("config_dir")
        .unwrap_or_else(|| default_config.as_str());
    if !(Path::new(&config_dir).exists()) {
        fs::create_dir_all(&config_dir).unwrap();
        created_dir = true;
    }
    let default_log_level = "info";
    let log_level = match matches.value_of("log_level").unwrap_or(&default_log_level) {
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Debug,
    };

    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            log_level,
            Config::default(),
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("{}/{}.log", config_dir, appname))
                .unwrap(),
        ),
    ])
    .unwrap();
    info!(
        "Logging started for v{} of {}, log level: {}",
        version, appname, log_level
    );
    if created_dir {
        info!("Created new config directory: {}", config_dir);
    }
}
