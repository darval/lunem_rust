use config::*;
use std::sync::RwLock;
use std::collections::HashMap;


lazy_static! {
    pub static ref SETTINGS: RwLock<Config> = RwLock::new({
        let mut settings = Config::default();
        settings.merge(File::with_name(&crate::CONFIGFILE.read().unwrap())).unwrap();
        let hm = settings.clone().try_into::<HashMap<String, String>>().unwrap();
        *crate::CONFIGDATA.write().unwrap() = hm;
        settings
    });
}