use js_sys::{Array, Uint8Array};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, JsValue};

use gloo_file::{callbacks::FileReader, File};
use wasm_bindgen::prelude::*;
use web_sys::{Blob, DragEvent, HtmlImageElement, HtmlInputElement, Url};
use yew::prelude::*;

use crate::image_container::{ImCanvasWrapper, ImType, ImageContainer};

use crate::PositionInfo;

use std::collections::HashMap;

pub struct App {
    image_loaded_closure: Closure<dyn FnMut(JsValue)>,
    image_error_closure: Closure<dyn FnMut(JsValue)>,
    readers: HashMap<String, FileReader>,
    file_info: Option<FileInfo>,
    im_orig: Rc<RefCell<ImCanvasWrapper>>,
    im_rotated: Rc<RefCell<ImCanvasWrapper>>,
    im_stretch: Rc<RefCell<ImCanvasWrapper>>,
    state: AppState,
    error_log: Vec<String>,
    position_info: Rc<RefCell<PositionInfo>>,
}

pub enum AppState {
    Ready,
    ReadingFile,
    DecodingImage(FileInfo),
}

pub struct FileInfo {
    file_data: gloo_file::File,
    // file_data: Vec<u8>,
    img: HtmlImageElement,
}

pub enum Msg {
    FileChanged(File),
    FileLoaded(String, Vec<u8>),
    // FileLoaded(FileData),
    // Files(Vec<File>),
    ImageLoaded,
    ImageErrored(String),
    FileDropped(DragEvent),
    FileDraggedOver(DragEvent),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link2 = ctx.link().clone();
        let image_loaded_closure = Closure::wrap(Box::new(move |_| {
            link2.send_message(Msg::ImageLoaded);
        }) as Box<dyn FnMut(JsValue)>);

        let link2 = ctx.link().clone();
        let image_error_closure = Closure::wrap(Box::new(move |arg| {
            // let err_str = format!("Failed to load image.{:?}", arg);
            let err_str = "Failed to load image.".into();
            log::error!("{:?}", arg);
            link2.send_message(Msg::ImageErrored(err_str));
        }) as Box<dyn FnMut(_)>);

        let position_info = Rc::new(RefCell::new(PositionInfo::new()));

        App {
            image_loaded_closure,
            image_error_closure,
            readers: Default::default(),
            im_orig: ImCanvasWrapper::new(ImType::Original, position_info.clone()),
            im_rotated: ImCanvasWrapper::new(ImType::Rotated, position_info.clone()),
            im_stretch: ImCanvasWrapper::new(ImType::Stretch, position_info.clone()),
            file_info: None,
            state: AppState::Ready,
            error_log: vec![],
            position_info,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        // fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        self.update_canvas_contents();
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ImageLoaded => {
                // The image has finished decoding and we can display it now.
                let old_state = std::mem::replace(&mut self.state, AppState::Ready);

                if let AppState::DecodingImage(file_info) = old_state {
                    self.position_info
                        .borrow_mut()
                        .update_for_image(&file_info.img);

                    self.file_info = Some(file_info);
                    self.update_canvas_contents();
                }
            }
            Msg::ImageErrored(err_str) => {
                // The image was not decoded due to an error.
                self.error_log.push(err_str);
                self.state = AppState::Ready;
            }
            Msg::FileLoaded(filename, rbuf) => {
                // The bytes of the file have been read.

                // Convert to a Uint8Array and initiate the image decoding.
                let buffer = Uint8Array::from(rbuf.as_slice());
                let buffer_val: &JsValue = buffer.as_ref();
                let parts = Array::new_with_length(1);
                parts.set(0, buffer_val.clone());
                let blob = Blob::new_with_u8_array_sequence(parts.as_ref()).unwrap();
                let img = HtmlImageElement::new().unwrap();

                img.set_onload(Some(self.image_loaded_closure.as_ref().unchecked_ref()));

                img.set_onerror(Some(self.image_error_closure.as_ref().unchecked_ref()));

                img.set_src(&Url::create_object_url_with_blob(&blob).unwrap());

                let file_data = gloo_file::File::new(filename.as_str(), rbuf.as_slice());

                self.state = AppState::DecodingImage(FileInfo { file_data, img });
            }
            // Msg::Files(files) => {
            //     // The user has selected file(s).
            //     self.error_log.clear();

            //     self.state = AppState::ReadingFile;

            //     for file in files.into_iter() {
            //         let task = {
            //             let callback = ctx.link().callback(Msg::FileLoaded);
            //             ReaderService::read_file(file, callback).unwrap()
            //         };
            //         self.tasks.push(task);
            //     }
            // }
            Msg::FileChanged(file) => {
                let filename = file.name();
                let link = ctx.link().clone();
                let filename2 = filename.clone();
                let reader = gloo_file::callbacks::read_as_bytes(&file, move |res| {
                    link.send_message(Msg::FileLoaded(
                        filename2,
                        res.expect("failed to read file"),
                    ))
                });
                self.readers.insert(filename, reader);
            }
            Msg::FileDropped(evt) => {
                evt.prevent_default();
                let files = evt.data_transfer().unwrap_throw().files();
                // log_1(&format!("files dropped: {:?}", files).into());
                if let Some(files) = files {
                    let mut result = Vec::new();
                    let files = js_sys::try_iter(&files)
                        .unwrap_throw()
                        .unwrap_throw()
                        .into_iter()
                        .map(|v| web_sys::File::from(v.unwrap_throw()))
                        .map(File::from);
                    result.extend(files);
                    assert!(result.len() == 1);
                    ctx.link()
                        .send_message(Msg::FileChanged(result.pop().unwrap_throw()));
                }
            }
            Msg::FileDraggedOver(evt) => {
                evt.prevent_default();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ondragover = ctx.link().callback(|e| Msg::FileDraggedOver(e));

        let ondrop = ctx.link().callback(|e| {
            // if let Some(ft) = e.data_transfer() {
            //     return Msg::Files(
            //         js_sys::try_iter(&ft.files().unwrap())
            //             .unwrap()
            //             .unwrap()
            //             .map(|v| File::from(v.unwrap()))
            //             .collect(),
            //     );
            // }

            Msg::FileDropped(e)
        });

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
                    <div class="drag-and-drop" ondrop={ondrop} ondragover={ondragover}>
                        {"Drag a file here or select an image."}
                        <label class={classes!("btn","file-btn")}>
                            <input
                                type="file"
                                accept="image/*"
                                multiple=false
                                onchange={ctx.link().callback(move |e: Event| {
                                    let mut result = Vec::new();
                                    let input: HtmlInputElement = e.target_unchecked_into();

                                    if let Some(files) = input.files() {
                                        let files = js_sys::try_iter(&files)
                                            .unwrap_throw()
                                            .unwrap_throw()
                                            .map(|v| web_sys::File::from(v.unwrap_throw()))
                                            .map(File::from);
                                        result.extend(files);
                                    }
                                    assert!(result.len()==1);
                                    Msg::FileChanged(result.pop().unwrap_throw())
                                    // Msg::Files(result)
                                })}/>
                            {"Select file..."}
                        </label>
                    </div>
                </div>

                { self.view_file_info() }
                <div id="hnb-app-canvas-div">
                    <h2><span class="stage">{"2"}</span>{"View the original, Color Stretched and Color Rotated images."}</h2>
                    <div id="hnb-app-canvas-container">
                        <ImageContainer canvas_wrapper={self.im_orig.clone()} />
                        <ImageContainer canvas_wrapper={self.im_stretch.clone()} />
                        <ImageContainer canvas_wrapper={self.im_rotated.clone()} />
                    </div>
                </div>
                { self.view_errors() }
            </div>
        }
    }
}

fn render_error(err_str: &str) -> Html {
    html! {
        <p>{format!("ERROR: {}", err_str)}</p>
    }
}

impl App {
    fn view_file_info(&self) -> Html {
        if let Some(file_info) = &self.file_info {
            html! {
                <p>{file_info.file_data.name().as_str()}</p>
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
    fn update_canvas_contents(&self) {
        if let Some(file_info) = &self.file_info {
            let mut im_orig = self.im_orig.borrow_mut();
            let fname = file_info.file_data.name().as_str();
            im_orig.draw_image(&file_info.img, fname);
            let image_data = im_orig.get_data();

            if let Some(image_data) = image_data {
                let mut im_rotated = self.im_rotated.borrow_mut();
                im_rotated.draw_data(&image_data, fname);

                let mut im_stretch = self.im_stretch.borrow_mut();
                im_stretch.draw_data(&image_data, fname);
            }
        }
    }
}
