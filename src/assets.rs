use std::{collections::HashMap, sync::OnceLock};

use aho_corasick::AhoCorasick;

use crate::config::Config;
use crate::PIPED_SRC;

static ASSETS: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn patch_assets(config: &Config) {
    // replace all matches to default backend isntance with new backend instance address
    // https://github.com/TeamPiped/Piped-Docker/blob/main/template/docker-compose.nginx.yml#L10
    let patterns = &["https://pipedapi.kavin.rocks"];
    let replace_with = &[&*config.addresses.backend];

    let ac = AhoCorasick::new(patterns).unwrap();

    let mut hashmap = HashMap::new();

    let dir = PIPED_SRC.get_dir("assets").unwrap();
    for file in dir.files() {
        if let Some(contents) = file.contents_utf8() {
            let replaced = ac.replace_all(contents, replace_with);
            hashmap.insert(file.path().to_str().unwrap().to_string(), replaced);
        }
    }

    ASSETS.set(hashmap).unwrap();
}

// gets a patched asset
pub fn get_patched_asset(target: &str) -> Option<&'static str> {
    ASSETS.get().unwrap().get(target).map(|i| &**i)
}
