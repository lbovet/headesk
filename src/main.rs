mod camera;
mod chromakey;
mod window;

use camera::CameraSwitcher;

fn main() {
    window::create(CameraSwitcher::new(0));
}
