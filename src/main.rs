#![windows_subsystem = "windows"]

#[macro_use]
mod camera;
mod chromakey;
mod window;

use camera::CameraSwitcher;
use preferences_ron::{AppInfo, Preferences};
use serde::{Deserialize, Serialize};
use window::View;

const APP_INFO: AppInfo = AppInfo {
    name: "headesk",
    author: "Headesk",
};

#[derive(Serialize, Deserialize, Debug)]
struct Prefs {
    views: Vec<View>,
}

fn main() {
    let prefs = Prefs::load(&APP_INFO, "user");

    window::create(CameraSwitcher::new(), prefs.map(|p| p.views[0]).ok(), |view| {
        Prefs { views: vec!(view) }.save(&APP_INFO, "user").unwrap();
    });
}
