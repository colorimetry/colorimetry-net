#![recursion_limit = "512"]

mod app;
mod image_container;
mod transform_colors;

use wasm_bindgen::prelude::*;

const TEXTBOX_HEIGHT_PX: i32 = 20;

pub struct PositionInfo {
    /// the dimension of the div containing both canvases, when known
    image_dims: Option<(u32, u32)>,
    canv_width: i32,
    image_height: i32,
    canv_height: i32,
}

impl PositionInfo {
    fn new() -> Self {
        let image_height = 200;
        let canv_height = image_height + TEXTBOX_HEIGHT_PX;
        Self {
            image_dims: None,
            canv_width: 300,
            image_height,
            canv_height,
        }
    }

    /// An image has been loaded, recalculate various sizing info.
    fn update_for_image(&mut self, img: &web_sys::HtmlImageElement) {
        log::info!("got image size {}x{}", img.width(), img.height());
        self.image_dims = Some((img.width(), img.height()));
        self.canv_width = img.width() as i32;
        self.image_height = img.height() as i32;
        self.canv_height = self.image_height + TEXTBOX_HEIGHT_PX;
    }

    /// The width of the canvas (canvas coords)
    fn canv_width(&self) -> i32 {
        self.canv_width
    }
    /// The height of the canvas (canvas coords)
    fn canv_height(&self) -> i32 {
        self.canv_height
    }
    /// The width of the canvas (canvas coords)
    fn canv_width_str(&self) -> String {
        format!("{}", self.canv_width)
    }
    /// The height of the canvas (canvas coords)
    fn canv_height_str(&self) -> String {
        format!("{}", self.canv_height)
    }
    /// The height of the image (canvas coords)
    fn image_height(&self) -> i32 {
        self.image_height
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is the entry point for the web app
#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    // yew::start_app::<app::App>();
    yew::initialize();
    let document = yew::utils::document();
    let div_wrapper: web_sys::Element = document.query_selector("#app-main").unwrap().unwrap();
    yew::app::App::<app::App>::new().mount(div_wrapper);
    yew::run_loop();
    Ok(())
}
