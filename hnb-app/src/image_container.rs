use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{Clamped, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{classes, html, Component, Context, Html, NodeRef, Properties};

use crate::PositionInfo;

const TEXT_PAD_PX: i32 = 2;
const FONT: &str = "16px sans-serif";

#[derive(Clone, Debug, PartialEq, Default)]
pub enum ImType {
    #[default]
    Original,
    Rotated,
    Stretch,
}

impl std::fmt::Display for ImType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImType::Original => write!(f, "Original"),
            ImType::Rotated => write!(f, "Color Rotated"),
            ImType::Stretch => write!(f, "Color Stretched"),
        }
    }
}

#[derive(PartialEq)]
pub struct ImCanvasWrapper {
    im_type: ImType,
    fname: String,
    context_2d: Option<CanvasRenderingContext2d>,
    canvas: Option<HtmlCanvasElement>,
    position_info: Rc<RefCell<PositionInfo>>,
}

impl Drop for ImCanvasWrapper {
    fn drop(&mut self) {
        log::info!("Dropping ImCanvasWrapper for {:?}", self.im_type);
    }
}

impl ImCanvasWrapper {
    pub fn new(im_type: ImType, position_info: Rc<RefCell<PositionInfo>>) -> Self {
        log::info!("Creating ImCanvasWrapper for {:?}", im_type);
        Self {
            im_type,
            fname: "".to_string(),
            context_2d: None,
            canvas: None,
            position_info,
        }
    }

    pub fn draw_image(&mut self, img: &web_sys::HtmlImageElement, fname: &str) {
        log::info!("ImCanvasWrapper::draw_image {}", fname);
        if let Some(ctx) = &self.context_2d {
            log::info!("  got context_2d");
            ctx.clear_rect(
                0.0,
                0.0,
                self.position_info.borrow().canv_width() as f64,
                self.position_info.borrow().canv_height() as f64,
            );

            // Draw the original image on the canvas.
            ctx.draw_image_with_html_image_element_and_dw_and_dh(
                img,
                0.0,
                0.0,
                self.position_info.borrow().canv_width() as f64,
                self.position_info.borrow().image_height() as f64,
            )
            .unwrap();

            let text = fname;
            self.fname = fname.to_string();
            self.draw_text(ctx, text);
        } else {
            log::error!("  no context_2d");
        }
    }

    pub fn draw_data(&mut self, image_data: &web_sys::ImageData, fname: &str) {
        log::info!("ImCanvasWrapper::draw_data {}", self.im_type);
        let mut data = image_data.data();
        match self.im_type {
            ImType::Original => {}
            ImType::Rotated => {
                crate::transform_colors::saturate_and_rotate(data.as_mut_slice());
            }
            ImType::Stretch => {
                crate::transform_colors::color_stretch(data.as_mut_slice());
            }
        }

        let w = image_data.width();
        let h = image_data.height();

        if let Some(ctx) = &self.context_2d {
            ctx.clear_rect(
                0.0,
                0.0,
                self.position_info.borrow().canv_width() as f64,
                self.position_info.borrow().canv_height() as f64,
            );

            let new_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                Clamped(data.as_mut_slice()),
                w,
                h,
            )
            .unwrap();
            ctx.put_image_data(&new_data, 0.0, 0.0).unwrap();

            let text = match self.im_type {
                ImType::Original => fname.to_string(),
                ImType::Rotated => format!("{fname}: Color Rotated"),
                ImType::Stretch => format!("{fname}: Color Stretched"),
            };
            self.fname = fname.to_string();
            self.draw_text(ctx, &text);
        }
    }

    fn draw_text(&self, ctx: &CanvasRenderingContext2d, text: &str) {
        ctx.set_text_baseline("top");
        ctx.set_font(FONT);
        ctx.fill_text_with_max_width(
            text,
            0.0,
            self.position_info.borrow().image_height() as f64 + TEXT_PAD_PX as f64,
            self.position_info.borrow().canv_width() as f64,
        )
        .unwrap();
    }

    pub fn get_data(&self) -> Option<web_sys::ImageData> {
        log::info!("ImCanvasWrapper::get_data {}", self.fname);
        if let Some(ctx) = &self.context_2d {
            let image_data: web_sys::ImageData = ctx
                .get_image_data(
                    0.0,
                    0.0,
                    self.position_info.borrow().canv_width() as f64,
                    self.position_info.borrow().image_height() as f64,
                )
                .unwrap();
            Some(image_data)
        } else {
            None
        }
    }

    fn basename(&self) -> String {
        let fname_os = std::ffi::OsString::from(&self.fname);

        let stem = std::path::Path::new(&self.fname)
            .file_stem()
            .unwrap_or(&fname_os);

        let what = match self.im_type {
            ImType::Original => "original",
            ImType::Rotated => "rotated",
            ImType::Stretch => "stretch",
        };

        format!("{}-{}", stem.to_str().unwrap(), what)
    }

    fn button_text(&self) -> &str {
        match self.im_type {
            ImType::Original => "Download original",
            ImType::Rotated => "Download color-rotated",
            ImType::Stretch => "Download color-stretched",
        }
    }
}

pub struct ImageContainer {
    node_ref: NodeRef,
}

pub enum Msg {
    Clicked,
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub im_type: ImType,
    pub canvas_wrapper: Rc<RefCell<ImCanvasWrapper>>,
    /// A count that changes when the image is updated, to force calling the
    /// ImageContainer::view() method to display the potentially new width and
    /// height of the HTML canvas element.
    pub count: u8,
}

impl Component for ImageContainer {
    type Message = Msg;
    type Properties = Props;
    fn create(ctx: &Context<Self>) -> Self {
        log::info!("Creating ImageContainer for {:?}", ctx.props().im_type);
        Self {
            node_ref: NodeRef::default(),
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        log::info!(
            "ImageContainer::rendered {}",
            ctx.props().canvas_wrapper.borrow().im_type
        );
        // Once rendered, store references for the canvas and 2D context. These can be used for
        // resizing the rendering area when the window or canvas element are resized.
        if ctx.props().canvas_wrapper.borrow().canvas.is_none() {
            log::info!(
                "  setting up canvas and context_2d for {:?}",
                ctx.props().canvas_wrapper.borrow().im_type
            );
            assert!(ctx.props().canvas_wrapper.borrow().context_2d.is_none());
            let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();

            let context = CanvasRenderingContext2d::from(JsValue::from(
                canvas.get_context("2d").unwrap().unwrap(),
            ));

            let canvas_wrapper = &mut ctx.props().canvas_wrapper.borrow_mut();
            canvas_wrapper.canvas.replace(canvas);
            canvas_wrapper.context_2d.replace(context);
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        log::info!(
            "ImageContainer::changed {}",
            ctx.props().canvas_wrapper.borrow().im_type
        );
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!(
            "ImageContainer::update {}",
            ctx.props().canvas_wrapper.borrow().im_type
        );
        match msg {
            Msg::Clicked => {
                let canvas_wrapper = ctx.props().canvas_wrapper.borrow();

                let data_url = canvas_wrapper
                    .canvas
                    .as_ref()
                    .unwrap()
                    .to_data_url_with_type("image/png")
                    .unwrap();

                let document = web_sys::window().unwrap().document().unwrap();

                let anchor = document
                    .create_element("a")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlAnchorElement>()
                    .unwrap();

                anchor.set_href(&data_url);
                let download_name = format!("{}.png", canvas_wrapper.basename());
                anchor.set_download(&download_name);
                anchor.set_target("_blank");

                anchor.style().set_property("display", "none").unwrap();
                let body = document.body().unwrap();
                body.append_child(&anchor).unwrap();

                anchor.click();

                body.remove_child(&anchor).unwrap();
                web_sys::Url::revoke_object_url(&data_url).unwrap();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cw = &ctx.props().canvas_wrapper.borrow();
        let btn_text = cw.button_text();
        let pi = &cw.position_info;
        let button = if !cw.fname.is_empty() {
            html! {
                <button
                    class={classes!("im-btn","btn")}
                    onclick={ctx.link().callback(|_| Msg::Clicked)}
                >
                    { btn_text }
                </button>
            }
        } else {
            html! {<span></span>}
        };
        let width = pi.borrow().canv_width_str();
        let height = pi.borrow().canv_height_str();
        log::info!(
            "ImageContainer::view {} {}x{}",
            ctx.props().im_type,
            width,
            height
        );
        html! {
            <span class="im-span">
                <div>
                    {button}
                </div>
                <div>
                    <canvas class="im-canvas" ref={&self.node_ref} width={width} height={height} />
                </div>
            </span>
        }
    }
}
