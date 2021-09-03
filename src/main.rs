mod camera;
mod window;

use camera::Camera;

fn main() {
    window::create(Camera::init(1));
}
