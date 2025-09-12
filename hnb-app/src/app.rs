use gloo_file::callbacks::FileReader;
use js_sys::{Array, Uint8Array};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, JsValue};
use web_sys::{Blob, HtmlImageElement, Url};
use yew::{html, Component, Context, Html, Properties};

use crate::image_container::{ImCanvasWrapper, ImType, ImageContainer};

use crate::{file_input::FileInput, PositionInfo};

pub struct App {
    readers: HashMap<String, FileReader>,
    file_info: Option<FileInfo>,
    im_orig: Rc<RefCell<ImCanvasWrapper>>,
    im_rotated: Rc<RefCell<ImCanvasWrapper>>,
    im_stretch: Rc<RefCell<ImCanvasWrapper>>,
    state: AppState,
    error_log: Vec<String>,
    /// A count that changes when the image is updated, to force calling the
    /// ImageContainer::view() method to use the potentially new width and
    /// height of the HTML canvas element.
    count: u8,
}

pub enum AppState {
    Ready,
    ReadingFile,
    DecodingImage(FileInfo),
}

pub struct FileData {
    content: Vec<u8>,
    name: String,
}

pub struct FileInfo {
    file_data: FileData,
    img: HtmlImageElement,
}

pub enum Msg {
    FileLoaded(FileData),
    Files(Vec<gloo_file::File>),
    ImageLoaded,
    ImageErrored(String),
}

#[derive(PartialEq, Properties)]
pub struct AppProps {
    pub position_info: Rc<RefCell<PositionInfo>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = AppProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            im_orig: Rc::new(RefCell::new(ImCanvasWrapper::new(
                ImType::Original,
                ctx.props().position_info.clone(),
            ))),
            im_rotated: Rc::new(RefCell::new(ImCanvasWrapper::new(
                ImType::Rotated,
                ctx.props().position_info.clone(),
            ))),
            im_stretch: Rc::new(RefCell::new(ImCanvasWrapper::new(
                ImType::Stretch,
                ctx.props().position_info.clone(),
            ))),
            file_info: None,
            state: AppState::Ready,
            error_log: vec![],
            readers: Default::default(),
            count: 0,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        self.update_canvas_contents();
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ImageLoaded => {
                log::debug!("Msg::ImageLoaded");
                // The image has finished decoding and we can display it now.
                let old_state = std::mem::replace(&mut self.state, AppState::Ready);

                if let AppState::DecodingImage(file_info) = old_state {
                    ctx.props()
                        .position_info
                        .borrow_mut()
                        .update_for_image(&file_info.img);

                    self.file_info = Some(file_info);
                    self.update_canvas_contents();
                }
            }
            Msg::ImageErrored(err_str) => {
                log::debug!("Msg::ImageErrored");
                // The image was not decoded due to an error.
                self.error_log.push(err_str);
                self.state = AppState::Ready;
            }
            Msg::FileLoaded(file_data) => {
                log::debug!("Msg::FileLoaded {}", file_data.name);
                // The bytes of the file have been read.

                // Convert to a Uint8Array and initiate the image decoding.
                let buffer = Uint8Array::from(file_data.content.as_slice());
                let buffer_val: &JsValue = buffer.as_ref();
                let parts = Array::new_with_length(1);
                parts.set(0, buffer_val.clone());
                let blob = Blob::new_with_u8_array_sequence(parts.as_ref()).unwrap();
                let img = HtmlImageElement::new().unwrap();

                // TODO: check that these callback are always received.

                // img load event
                let callback = ctx.link().callback(move |_| Msg::ImageLoaded);

                let on_load_closure = Closure::wrap(Box::new(move || {
                    callback.emit(()); // dummy arg for callback
                }) as Box<dyn FnMut()>);

                img.set_onload(Some(on_load_closure.as_ref().unchecked_ref()));
                on_load_closure.forget();

                // img error event
                let callback = ctx.link().callback(move |arg| {
                    log::error!("{:?}", arg);
                    Msg::ImageErrored("Failed to load image.".into())
                });

                let on_error_closure = Closure::wrap(Box::new(move || {
                    callback.emit(()); // dummy arg for callback
                }) as Box<dyn FnMut()>);

                img.set_onerror(Some(on_error_closure.as_ref().unchecked_ref()));
                on_error_closure.forget();

                // img set source
                img.set_src(&Url::create_object_url_with_blob(&blob).unwrap());

                self.state = AppState::DecodingImage(FileInfo { file_data, img });
            }
            Msg::Files(files) => {
                // The user has selected file(s).
                self.error_log.clear();

                for file in files.into_iter() {
                    log::debug!("Msg::Files: file {}", file.name());
                    let file_name = file.name();
                    let task = {
                        let file_name = file_name.clone();
                        let link = ctx.link().clone();
                        gloo_file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::FileLoaded(FileData {
                                name: file_name,
                                content: res.expect("failed to read file"),
                            }))
                        })
                    };
                    self.readers.insert(file_name, task);
                }

                self.state = AppState::ReadingFile;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (state, spinner_div_class) = match self.state {
            AppState::Ready => ("Ready", "display-none"),
            AppState::ReadingFile => ("Reading file", "compute-modal"),
            AppState::DecodingImage(_) => ("Decoding image", "compute-modal"),
        };

        // Hmm, on iOS we do not get the original image but a lower quality
        // version converted to JPEG:
        // https://stackoverflow.com/q/27673102/1633026

        html! {
            <div class="spa-container">
                <div>
                <h3>{"Color Stretch"}</h3>
                <p>{"In a Hue-Saturation-Lightness colorspace, the color of each pixel will be \
                stretched in hue to emphasize the colors of HNB and increased 4x in saturation. \
                This increases the perceptual ability to distinguish positive vs \
                negative outcomes of SARS-CoV-2 tests using an isothermal LAMP reaction with \
                HNB (Hydroxy naphthol blue) dye."}</p>
                <h3>{"Color Rotate"}</h3>
                <p>{"In a Hue-Saturation-Lightness colorspace, the color of each pixel will be \
                increased 4x in saturation and rotated 180 degrees in Hue. "}
                <a href="https://doi.org/10.1101/2020.06.23.166397 ">{"Kellner et al. (2020)"}</a>
                {" found that this increases the perceptual ability to distinguish positive vs \
                negative outcomes of SARS-CoV-2 tests using an isothermal LAMP reaction with \
                HNB (Hydroxy naphthol blue) dye."}</p>
                </div>
                <div class={spinner_div_class}>
                    <div class="compute-modal-inner">
                        <p>
                            {state}
                        </p>
                        <div class="lds-ellipsis">
                            <div></div><div></div><div></div><div></div>
                        </div>
                    </div>
                </div>
                <div>
                    <h2><span class="stage">{"1"}</span>{"Choose an image file."}</h2>
                    <div class="drag-and-drop" >
                        {"Drag a file here or select an image."}

                        <FileInput
                            button_text={"Select file..."}
                            multiple=false
                            accept={"image/*"}
                            on_changed={ctx.link().callback(|files| {
                                Msg::Files(files)
                            })}
                        />
                    </div>
                </div>

                { self.view_file_info() }
                <div id="hnb-app-canvas-div">
                    <h2><span class="stage">{"2"}</span>{"View the original, Color Stretched and Color Rotated images."}</h2>
                    <div id="hnb-app-canvas-container">
                        <ImageContainer count={self.count} im_type={ImType::Original} canvas_wrapper={self.im_orig.clone()}/>
                        <ImageContainer count={self.count} im_type={ImType::Rotated} canvas_wrapper={self.im_rotated.clone()}/>
                        <ImageContainer count={self.count} im_type={ImType::Stretch} canvas_wrapper={self.im_stretch.clone()}/>
                    </div>
                </div>
                { self.view_errors() }
            </div>
        }
    }
}

fn render_error(err_str: &str) -> Html {
    html! {
        <p>{format!("ERROR: {err_str}")}</p>
    }
}

impl App {
    fn view_file_info(&self) -> Html {
        if let Some(file_info) = &self.file_info {
            html! {
                <p>{file_info.file_data.name.as_str()}</p>
            }
        } else {
            html! {}
        }
    }

    fn view_errors(&self) -> Html {
        if self.error_log.is_empty() {
            html! {}
        } else {
            html! {
                <div>
                    { for self.error_log.iter().map(String::as_str).map(render_error)}
                </div>
            }
        }
    }

    /// Redraw the canvases.
    ///
    /// We need to do this either when we get a new image decoded or
    /// when the dimensions of our container change.
    fn update_canvas_contents(&mut self) {
        log::debug!("App::update_canvas_contents");
        if let Some(file_info) = &self.file_info {
            let im_orig = &mut self.im_orig;
            let fname = file_info.file_data.name.as_str();
            im_orig.borrow_mut().draw_image(&file_info.img, fname);
            let image_data = im_orig.borrow().get_data();

            if let Some(image_data) = image_data {
                log::debug!("App::update_canvas_contents got image data");
                let im_rotated = &mut self.im_rotated;
                im_rotated.borrow_mut().draw_data(&image_data, fname);

                let im_stretch = &mut self.im_stretch;
                im_stretch.borrow_mut().draw_data(&image_data, fname);
            }

            // Force ImageContainer::view() to be called.
            self.count = self.count.wrapping_add(1);
        }
    }
}
