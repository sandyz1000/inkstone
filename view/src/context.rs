use std::rc::Rc;

use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{ Vector2F, Vector2I };

use crate::config::{ Config, Icon };

pub trait ViewBackend {
    fn resize(&mut self, size: Vector2F);
    fn get_scroll_factors(&self) -> (Vector2F, Vector2F);
    fn set_icon(&mut self, icon: Icon);
}

pub struct Context<B: ViewBackend> {
    // - the window needs a repaint
    pub redraw_requested: bool,
    pub page_nr: usize,
    pub num_pages: usize,
    pub scale: f32, // device independend
    pub view_center: Vector2F,
    pub window_size: Vector2F, // in pixels
    pub scale_factor: f32, // device dependend
    pub config: Rc<Config>,
    pub bounds: Option<RectF>,
    pub close: bool,
    pub update_interval: Option<f32>,
    pub pixel_scroll_factor: Vector2F,
    pub line_scroll_factor: Vector2F,
    pub backend: B,
}

pub const DEFAULT_SCALE: f32 = 96.0 / 25.4;

impl<'a, B: ViewBackend> Context<B> {
    pub fn new(config: Rc<Config>, backend: B) -> Self {
        let (pixel_scroll_factor, line_scroll_factor) = backend.get_scroll_factors();
        Context {
            redraw_requested: true,
            num_pages: 1,
            page_nr: 0,
            scale: DEFAULT_SCALE,
            scale_factor: 1.0,
            config: config.clone(),
            view_center: Vector2F::default(),
            window_size: Vector2F::default(),
            bounds: None,
            close: false,
            update_interval: None,
            pixel_scroll_factor,
            line_scroll_factor,
            backend,
        }
    }

    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    pub fn goto_page(&mut self, page: usize) {
        let page = page.min(self.num_pages - 1);
        if page != self.page_nr {
            self.page_nr = page;
            self.request_redraw();
        }
    }

    pub fn next_page(&mut self) {
        self.goto_page(self.page_nr.saturating_add(1));
    }

    pub fn prev_page(&mut self) {
        self.goto_page(self.page_nr.saturating_sub(1));
    }

    pub fn page_nr(&self) -> usize {
        self.page_nr
    }

    pub fn zoom_by(&mut self, log2_factor: f32) {
        self.scale *= (2f32).powf(log2_factor);
        self.check_bounds();
        self.request_redraw();
    }

    pub fn set_zoom(&mut self, factor: f32) {
        if factor != self.scale {
            self.scale = factor;
            self.check_bounds();
            self.request_redraw();
        }
    }

    pub fn close(&mut self) {
        self.close = true;
    }

    pub fn move_by(&mut self, delta: Vector2F) {
        self.move_to(self.view_center + delta);
    }

    pub fn check_bounds(&mut self) {
        if let Some(bounds) = self.bounds {
            let mut point = self.view_center;
            // scale window size
            let ws = self.window_size * (1.0 / self.scale);

            if ws.x() >= bounds.width() {
                // center horizontally
                point.set_x(bounds.origin_x() + bounds.width() * 0.5);
            } else {
                let x = point.x();
                let x = x.max(bounds.origin_x() + ws.x() * 0.5);
                let x = x.min(bounds.origin_x() + bounds.width() - ws.x() * 0.5);
                point.set_x(x);
            }
            if ws.y() >= bounds.height() {
                // center vertically
                point.set_y(bounds.origin_y() + bounds.height() * 0.5);
            } else {
                let y = point.y();
                let y = y.max(bounds.origin_y() + ws.y() * 0.5);
                let y = y.min(bounds.origin_y() + bounds.height() - ws.y() * 0.5);
                point.set_y(y);
            }
            self.view_center = point;
        }
    }

    pub fn move_to(&mut self, point: Vector2F) {
        self.view_center = point;
        self.check_bounds();
        self.request_redraw();
    }

    pub fn set_bounds(&mut self, bounds: RectF) {
        self.bounds = Some(bounds);
        self.check_bounds();
    }

    pub fn set_scale_factor(&mut self, factor: f32) {
        self.scale_factor = factor;
        self.check_bounds();
        self.request_redraw();
    }

    pub fn window_size(&self) -> Vector2F {
        self.window_size
    }

    pub fn set_window_size(&mut self, size: Vector2F) {
        self.window_size = size;
        self.backend.resize(size);

        self.check_bounds();
        self.request_redraw();
    }

    pub fn view_transform(&self) -> Transform2F {
        Transform2F::from_translation(self.window_size * 0.5) *
            Transform2F::from_scale(self.scale) *
            Transform2F::from_translation(-self.view_center)
    }

    pub fn set_view_box(&mut self, view_box: RectF) {
        self.window_size = view_box.size();
        self.check_bounds();
        self.sanity_check();
        self.request_redraw();
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.check_bounds();
    }

    fn sanity_check(&mut self) {
        let max_window_size = Vector2F::new(500.0, 500.0);
        let s = (Vector2F::splat(1.0) / self.window_size) * max_window_size;
        self.scale *= (1f32).min(s.x()).min(s.y());
        self.window_size *= s;
    }

    pub fn send(&mut self, data: Vec<u8>) {}

    pub fn set_icon(&mut self, icon: Icon) {
        self.backend.set_icon(icon);
    }
}
