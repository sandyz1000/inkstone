// This is taken from >>>
// https://github.com/s3bk/pathfinder_view/blob/master/src/lib.rs

use pathfinder_color::ColorF;
use pathfinder_geometry::rect::RectF;

use pathfinder_renderer::gpu::options::RendererLevel;
use pathfinder_renderer::scene::Scene;
use pathfinder_resources::ResourceLoader;

pub struct Config {
    pub zoom: bool,
    pub pan: bool,
    pub borders: bool,
    pub transparent: bool,
    pub background: ColorF,
    pub render_level: RendererLevel,
    pub resource_loader: Box<dyn ResourceLoader>,
    pub threads: bool,
}

impl Config {
    pub fn new(resource_loader: Box<dyn ResourceLoader>) -> Self {
        Config {
            zoom: true,
            pan: true,
            borders: true,
            transparent: false,
            background: ColorF::white(),
            render_level: RendererLevel::D3D9,
            resource_loader,
            threads: true,
        }
    }
}

pub struct Icon {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl From<image::RgbaImage> for Icon {
    fn from(img: image::RgbaImage) -> Icon {
        let (width, height) = img.dimensions();
        let data = img.into_vec();
        Icon {
            width,
            height,
            data,
        }
    }
}

pub fn view_box(scene: &Scene) -> RectF {
    let view_box = scene.view_box();
    if view_box == RectF::default() {
        scene.bounds()
    } else {
        view_box
    }
}
