use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, Clamped, JsValue};
use web_sys::{Blob, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, Url};
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::{html, ChangeData, Component, ComponentLink, Html, NodeRef, ShouldRender};

pub struct App {
    link: ComponentLink<Self>,
    image_loaded_closure: Closure<dyn FnMut(JsValue)>,
    image_error_closure: Closure<dyn FnMut(JsValue)>,
    tasks: Vec<ReaderTask>,
    file_info: Option<FileInfo>,
    node_ref: NodeRef,
    context_2d: Option<CanvasRenderingContext2d>,
    canvas: Option<HtmlCanvasElement>,
    state: AppState,
    error_log: Vec<String>,
    image_canv_width: usize,
    canv_height: usize,
}

pub enum AppState {
    Ready,
    ReadingFile,
    DecodingImage(FileInfo),
}

pub struct FileInfo {
    file_data: FileData,
    img: HtmlImageElement,
}

pub enum Msg {
    FileLoaded(FileData),
    Files(Vec<File>),
    ImageLoaded,
    ImageErrored(String),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link2 = link.clone();
        let image_loaded_closure = Closure::wrap(Box::new(move |_| {
            link2.send_message(Msg::ImageLoaded);
        }) as Box<dyn FnMut(JsValue)>);

        let link2 = link.clone();
        let image_error_closure = Closure::wrap(Box::new(move |arg| {
            // let err_str = format!("Failed to load image.{:?}", arg);
            let err_str = "Failed to load image.".into();
            log::error!("{:?}", arg);
            link2.send_message(Msg::ImageErrored(err_str));
        }) as Box<dyn FnMut(_)>);

        App {
            link,
            image_loaded_closure,
            image_error_closure,
            tasks: vec![],
            node_ref: NodeRef::default(),
            file_info: None,
            context_2d: None,
            canvas: None,
            state: AppState::Ready,
            error_log: vec![],
            image_canv_width: 300,
            canv_height: 200,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn rendered(&mut self, _first_render: bool) {
        // Once rendered, store references for the canvas and 2D context. These can be used for
        // resizing the rendering area when the window or canvas element are resized.

        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();

        let context_2d = CanvasRenderingContext2d::from(JsValue::from(
            canvas.get_context("2d").unwrap().unwrap(),
        ));

        self.canvas = Some(canvas);
        self.context_2d = Some(context_2d);
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ImageLoaded => {
                let old_state = std::mem::replace(&mut self.state, AppState::Ready);

                if let AppState::DecodingImage(file_info) = old_state {
                    // The image has finished loading (decoding).
                    if let Some(ctx) = self.context_2d.as_ref() {
                        // ctx.draw_image_with_html_image_element(&file_info.img, 0.0, 0.0)
                        //     .unwrap();

                        ctx.draw_image_with_html_image_element_and_dw_and_dh(
                            &file_info.img,
                            0.0,
                            0.0,
                            self.image_canv_width as f64,
                            self.canv_height as f64,
                        )
                        .unwrap();

                        let image_data: web_sys::ImageData = ctx
                            .get_image_data(
                                0.0,
                                0.0,
                                self.image_canv_width as f64,
                                self.canv_height as f64,
                            )
                            .unwrap();

                        let w = image_data.width();
                        let h = image_data.height();

                        let mut data = image_data.data();
                        let rgba: &mut [u8] = data.as_mut_slice();

                        // set first 10 rows clear
                        for i in 0..(4 * 10 * w as usize) {
                            rgba[i] = 0;
                        }

                        let new_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                            Clamped(rgba),
                            w,
                            h,
                        )
                        .unwrap();

                        ctx.put_image_data(&new_data, self.image_canv_width as f64, 0.0)
                            .unwrap();
                    }
                    self.file_info = Some(file_info);
                }
            }
            Msg::ImageErrored(err_str) => {
                self.error_log.push(err_str);
                self.state = AppState::Ready;
            }
            Msg::FileLoaded(file_data) => {
                let buffer = Uint8Array::from(file_data.content.as_slice());
                let buffer_val: &JsValue = buffer.as_ref();
                let parts = Array::new_with_length(1);
                parts.set(0, buffer_val.clone());
                let blob = Blob::new_with_u8_array_sequence(parts.as_ref()).unwrap();
                let img = HtmlImageElement::new().unwrap();

                img.set_onload(Some(self.image_loaded_closure.as_ref().unchecked_ref()));

                img.set_onerror(Some(self.image_error_closure.as_ref().unchecked_ref()));

                img.set_src(&Url::create_object_url_with_blob(&blob).unwrap());

                self.state = AppState::DecodingImage(FileInfo { file_data, img });
            }
            Msg::Files(files) => {
                self.error_log.clear();

                self.state = AppState::ReadingFile;

                for file in files.into_iter() {
                    let task = {
                        let callback = self.link.callback(Msg::FileLoaded);
                        ReaderService::read_file(file, callback).unwrap()
                    };
                    self.tasks.push(task);
                }
            }
        }
        true
    }

    fn view(&self) -> Html {
        let state = match self.state {
            AppState::Ready => "Ready",
            AppState::ReadingFile => "Reading file",
            AppState::DecodingImage(_) => "Decoding image",
        };
        html! {
            <div class="colorswitch-wrapper">
                <section class="main">
                    <div>
                        <p>{ state }</p>
                        <p>{"Choose an image file to colorswitch."}</p>
                        <input type="file" onchange=self.link.callback(move |value| {
                                let mut result = Vec::new();
                                if let ChangeData::Files(files) = value {
                                    let files = js_sys::try_iter(&files)
                                        .unwrap()
                                        .unwrap()
                                        .into_iter()
                                        .map(|v| File::from(v.unwrap()));
                                    result.extend(files);
                                }
                                Msg::Files(result)
                            })/>
                    </div>

                    { self.view_file_info() }
                    <canvas ref={self.node_ref.clone()}, width={self.image_canv_width*2}, height={self.canv_height} />
                    { self.view_errors() }

                </section>

                <footer class="info">
                    <p>{ "Source code " }<a href="https://github.com/strawlab/colorswitch">{ "github.com/strawlab/colorswitch" }</a></p>
                </footer>
            </div>
        }
    }
}

fn render_error(err_str: &String) -> Html {
    html! {
        <p>{format!("ERROR: {}",err_str)}</p>
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
        if self.error_log.len() > 0 {
            html! {
                <div>
                    { for self.error_log.iter().map(render_error)}
                </div>
            }
        } else {
            html! {}
        }
    }
}
