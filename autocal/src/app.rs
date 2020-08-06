use js_sys::{Array, Uint8Array};
use palette::Pixel;
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, Clamped, JsValue};
use web_sys::{
    Blob, CanvasRenderingContext2d, DragEvent, HtmlCanvasElement, HtmlImageElement, Url,
};
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::{html, ChangeData, Component, ComponentLink, Html, NodeRef, ShouldRender};

use git_version::git_version;
const GIT_VERSION: &str = git_version!();

pub struct App {
    link: ComponentLink<Self>,
    c1_node_ref: NodeRef,
    c1_context_2d: Option<CanvasRenderingContext2d>,
    c1_canvas: Option<HtmlCanvasElement>,
    position_info: PositionInfo,
}

pub struct PositionInfo {
    /// the dimension of the div containing both canvases, when known
    image_dims: Option<(u32, u32)>,
    canv_width: i32,
    canv_height: i32,
}

impl PositionInfo {
    fn new() -> Self {
        Self {
            image_dims: None,
            canv_width: 300,
            canv_height: 200,
        }
    }

    /// An image has been loaded, recalculate various sizing info.
    fn update_for_image(&mut self, img: &HtmlImageElement) {
        log::info!("got image size {}x{}", img.width(), img.height());
        self.image_dims = Some((img.width(), img.height()));
        self.canv_width = img.width() as i32;
        self.canv_height = img.height() as i32;
    }

    /// The width of the canvas (canvas coords)
    fn canv_width(&self) -> i32 {
        self.canv_width
    }
    /// The height of the canvas (canvas coords)
    fn canv_height(&self) -> i32 {
        self.canv_height
    }
}

pub enum Msg {
    Nop,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link,
            c1_node_ref: NodeRef::default(),
            c1_context_2d: None,
            c1_canvas: None,
            position_info: PositionInfo::new(),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn rendered(&mut self, _first_render: bool) {
        // Once rendered, store references for the canvas and 2D context. These can be used for
        // resizing the rendering area when the window or canvas element are resized.

        self.update_canvas_contents();

        let canvas = self.c1_node_ref.cast::<HtmlCanvasElement>().unwrap();

        let context_2d = CanvasRenderingContext2d::from(JsValue::from(
            canvas.get_context("2d").unwrap().unwrap(),
        ));

        self.c1_canvas = Some(canvas);
        self.c1_context_2d = Some(context_2d);
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Nop => return false,
        }
        // true
    }

    fn view(&self) -> Html {
        let git_rev_link = format!(
            "https://github.com/colorimetry/colorimetry-net/commit/{}",
            GIT_VERSION
        );

        html! {
            <div class="spa-container">

                <canvas class="im-canv" ref={self.c1_node_ref.clone()}, width={self.position_info.canv_width()}, height={self.position_info.canv_height()} />

                <div>
                    <p>{"You are using revision "}<a href={git_rev_link}>{GIT_VERSION}</a>{"."}</p>
                </div>
            </div>
        }
    }
}

impl App {
    /// Redraw the canvas
    fn update_canvas_contents(&self) {
        if let Some(ctx1) = self.c1_context_2d.as_ref() {
            log::info!("drawing canvas images");

            // Draw the original image on the canvas.
            // ctx1.draw_image_with_html_image_element_and_dw_and_dh(
            //     &file_info.img,
            //     0.0,
            //     0.0,
            //     self.position_info.canv_width() as f64,
            //     self.position_info.canv_height() as f64,
            // )
            // .unwrap();

            // // Read the original image data from the canvas.
            // let image_data: web_sys::ImageData = ctx1
            //     .get_image_data(
            //         0.0,
            //         0.0,
            //         self.position_info.canv_width() as f64,
            //         self.position_info.canv_height() as f64,
            //     )
            //     .unwrap();

            // let w = image_data.width();
            // let h = image_data.height();
            // debug_assert!(w as i32 == self.position_info.canv_width());
            // debug_assert!(h as i32 == self.position_info.canv_height());
        }
    }
}
