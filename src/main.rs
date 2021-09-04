mod camera;
mod window;
mod chromakey;

use camera::Camera;

fn main() {
    window::create(Camera::init(0));
}
