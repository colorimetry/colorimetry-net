use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{Clamped, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::{html, Component, ComponentLink, Html, NodeRef, Properties, ShouldRender};

use crate::PositionInfo;

const TEXT_PAD_PX: i32 = 2;
const FONT: &str = "16px sans-serif";

#[derive(Clone, Debug)]
pub enum ImType {
    Original,
    Rotated,
    Stretch,
}

pub struct ImCanvasWrapper {
    im_type: ImType,
    fname: String,
    context_2d: Option<CanvasRenderingContext2d>,
    canvas: Option<HtmlCanvasElement>,
    position_info: Rc<RefCell<PositionInfo>>,
}

impl ImCanvasWrapper {
    pub fn new(im_type: ImType, position_info: Rc<RefCell<PositionInfo>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            im_type,
            fname: "".to_string(),
            context_2d: None,
            canvas: None,
            position_info,
        }))
    }
}

impl ImCanvasWrapper {
    pub fn draw_image(&mut self, img: &web_sys::HtmlImageElement, fname: &str) {
        if let Some(ctx) = &self.context_2d {
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
        }
    }

    pub fn draw_data(&mut self, image_data: &web_sys::ImageData, fname: &str) {
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
                ImType::Rotated => format!("{}: Color Rotated", fname),
                ImType::Stretch => format!("{}: Color Stretched", fname),
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
    link: ComponentLink<Self>,
    node_ref: NodeRef,
    canvas_wrapper: Rc<RefCell<ImCanvasWrapper>>,
}

pub enum Msg {
    Clicked,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub canvas_wrapper: Rc<RefCell<ImCanvasWrapper>>,
}

impl Component for ImageContainer {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            node_ref: NodeRef::default(),
            canvas_wrapper: props.canvas_wrapper,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.canvas_wrapper = props.canvas_wrapper;
        true
    }

    fn rendered(&mut self, _first_render: bool) {
        // Once rendered, store references for the canvas and 2D context. These can be used for
        // resizing the rendering area when the window or canvas element are resized.
        if self.canvas_wrapper.borrow().canvas.is_none() {
            assert!(self.canvas_wrapper.borrow().context_2d.is_none());
            let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();

            let context = CanvasRenderingContext2d::from(JsValue::from(
                canvas.get_context("2d").unwrap().unwrap(),
            ));

            let mut canvas_wrapper = self.canvas_wrapper.borrow_mut();
            canvas_wrapper.canvas.replace(canvas);
            canvas_wrapper.context_2d.replace(context);
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Clicked => {
                let canvas_wrapper = self.canvas_wrapper.borrow();

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
        false
    }

    fn view(&self) -> Html {
        let cw = self.canvas_wrapper.borrow();
        let btn_text = cw.button_text();
        let pi = cw.position_info.borrow();
        let button = if cw.fname.is_empty() {
            html! {
                <button class=("im-btn","btn"), onclick=self.link.callback(|_| Msg::Clicked)>{ btn_text }</button>
            }
        } else {
            html! {<span></span>}
        };
        html! {
            <span class="im-span">
                <div>
                    {button}
                </div>
                <div>
                    <canvas class="im-canvas", ref={self.node_ref.clone()}, width={pi.canv_width()}, height={pi.canv_height()} />
                </div>
            </span>
        }
    }
}
