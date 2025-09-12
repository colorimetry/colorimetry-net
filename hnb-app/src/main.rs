#![recursion_limit = "512"]

mod app;
mod file_input;
mod image_container;
mod transform_colors;

use console_error_panic_hook::set_once as set_panic_hook;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

use crate::app::AppProps;

const TEXTBOX_HEIGHT_PX: i32 = 20;

#[derive(PartialEq)]
pub struct PositionInfo {
    /// the dimension of the div containing both canvases, when known
    image_dims: Option<(u32, u32)>,
    canv_width: i32,
    image_height: i32,
    canv_height: i32,
}

impl Default for PositionInfo {
    fn default() -> Self {
        let image_height = 200;
        let canv_height = image_height + TEXTBOX_HEIGHT_PX;
        Self {
            image_dims: None,
            canv_width: 300,
            image_height,
            canv_height,
        }
    }
}

impl PositionInfo {
    /// An image has been loaded, recalculate various sizing info.
    fn update_for_image(&mut self, img: &web_sys::HtmlImageElement) {
        log::debug!("got image size {}x{}", img.width(), img.height());
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

fn main() -> Result<(), JsValue> {
    set_panic_hook();
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let div_wrapper: web_sys::Element = document.query_selector("#app-main").unwrap().unwrap();

    yew::Renderer::<crate::app::App>::with_root_and_props(
        div_wrapper,
        AppProps {
            position_info: Rc::new(RefCell::new(PositionInfo::default())),
        },
    )
    .render();
    Ok(())
}
