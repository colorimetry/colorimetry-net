use js_sys::{Array, Uint8Array};
use palette::Pixel;
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, Clamped, JsValue};
use web_sys::{Blob, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, Url};
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::{html, ChangeData, Component, ComponentLink, Html, NodeRef, ShouldRender};

use git_version::git_version;
const GIT_VERSION: &str = git_version!();

pub struct App {
    link: ComponentLink<Self>,
    image_loaded_closure: Closure<dyn FnMut(JsValue)>,
    image_error_closure: Closure<dyn FnMut(JsValue)>,
    tasks: Vec<ReaderTask>,
    file_info: Option<FileInfo>,
    c1_node_ref: NodeRef,
    c1_context_2d: Option<CanvasRenderingContext2d>,
    c1_canvas: Option<HtmlCanvasElement>,
    c2_node_ref: NodeRef,
    c2_context_2d: Option<CanvasRenderingContext2d>,
    c2_canvas: Option<HtmlCanvasElement>,
    state: AppState,
    error_log: Vec<String>,
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
            c1_node_ref: NodeRef::default(),
            c1_context_2d: None,
            c1_canvas: None,
            c2_node_ref: NodeRef::default(),
            c2_context_2d: None,
            c2_canvas: None,
            file_info: None,
            state: AppState::Ready,
            error_log: vec![],
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

        let canvas = self.c2_node_ref.cast::<HtmlCanvasElement>().unwrap();

        let context_2d = CanvasRenderingContext2d::from(JsValue::from(
            canvas.get_context("2d").unwrap().unwrap(),
        ));

        self.c2_canvas = Some(canvas);
        self.c2_context_2d = Some(context_2d);
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ImageLoaded => {
                // The image has finished decoding and we can display it now.
                let old_state = std::mem::replace(&mut self.state, AppState::Ready);

                if let AppState::DecodingImage(file_info) = old_state {
                    self.position_info.update_for_image(&file_info.img);
                    self.file_info = Some(file_info);
                    self.update_canvas_contents();
                }
            }
            Msg::ImageErrored(err_str) => {
                // The image was not decoded due to an error.
                self.error_log.push(err_str);
                self.state = AppState::Ready;
            }
            Msg::FileLoaded(file_data) => {
                // The bytes of the file have been read.

                // Convert to a Uint8Array and initiate the image decoding.
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
                // The user has selected file(s).
                self.error_log.clear();

                self.state = AppState::ReadingFile;

                for file in files.into_iter() {
                    let task = {
                        let callback = self.link.callback(Msg::FileLoaded);
                        ReaderService::new().read_file(file, callback).unwrap()
                    };
                    self.tasks.push(task);
                }
            }
        }
        true
    }

    fn view(&self) -> Html {
        let (state, spinner_div_class) = match self.state {
            AppState::Ready => ("Ready", "display-none"),
            AppState::ReadingFile => ("Reading file", "compute-modal"),
            AppState::DecodingImage(_) => ("Decoding image", "compute-modal"),
        };
        let git_rev_link = format!(
            "https://github.com/colorimetry/colorimetry-net/commit/{}",
            GIT_VERSION
        );

        // Hmm, on iOS we do not get the original image but a lower quality
        // version converted to JPEG:
        // https://stackoverflow.com/q/27673102/1633026

        html! {
            <div class="container">
                <div class=(spinner_div_class),>
                    <div class="compute-modal-inner",>
                        <p>
                            {state}
                        </p>
                        <div class="lds-ellipsis",>
                            <div></div><div></div><div></div><div></div>
                        </div>
                    </div>
                </div>
                <div>
                    <h2><span class="stage">{"1"}</span>{"Choose an image file."}</h2>
                    <input type="file" accept="image/*" onchange=self.link.callback(move |value| {
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
                <div id="colorimetry-net-canvas-div">
                    <h2><span class="stage">{"2"}</span>{"View the original and ColorSwitched image."}</h2>
                    <div id="colorimetry-net-canvas-container">
                        <canvas class="im-canv" ref={self.c1_node_ref.clone()}, width={self.position_info.canv_width()}, height={self.position_info.canv_height()} />
                        <canvas class="im-canv" ref={self.c2_node_ref.clone()}, width={self.position_info.canv_width()}, height={self.position_info.canv_height()} />
                    </div>
                </div>
                { self.view_errors() }

                // <div>
                //     <h2><span class="stage">{"3"}</span>{"Quantify your results."}</h2>
                //     {"To be implemented..."}
                // </div>

                <div>
                    <p>{"You are using revision "}<a href={git_rev_link}>{GIT_VERSION}</a>{"."}</p>
                </div>
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
    fn update_canvas_contents(&self) {
        if let Some(file_info) = &self.file_info {
            // The image has finished loading (decoding).
            if let (Some(ctx1), Some(ctx2)) =
                (self.c1_context_2d.as_ref(), self.c2_context_2d.as_ref())
            {
                log::info!("drawing canvas images");

                // Draw the original image on the canvas.
                ctx1.draw_image_with_html_image_element_and_dw_and_dh(
                    &file_info.img,
                    0.0,
                    0.0,
                    self.position_info.canv_width() as f64,
                    self.position_info.canv_height() as f64,
                )
                .unwrap();

                // Read the original image data from the canvas.
                let image_data: web_sys::ImageData = ctx1
                    .get_image_data(
                        0.0,
                        0.0,
                        self.position_info.canv_width() as f64,
                        self.position_info.canv_height() as f64,
                    )
                    .unwrap();

                let w = image_data.width();
                let h = image_data.height();
                debug_assert!(w as i32 == self.position_info.canv_width());
                debug_assert!(h as i32 == self.position_info.canv_height());

                let new_data = {
                    let mut data = image_data.data();

                    // Technically, it is probably wrong to load as linear, as
                    // the images are likely in sRGB colorspace. However, this
                    // gives a better match to the results (visually inspected)
                    // of operations with "Color Inspector 3D" by Kai Uwe
                    // Barthel. Furthermore, apparently [it is not specified
                    // what colorspace browsers use to draw images in the canvas
                    // element](https://wiki.whatwg.org/wiki/CanvasColorSpace).

                    let color_buffer: &mut [palette::rgb::Rgba<
                        palette::encoding::Linear<_>,
                        u8,
                    >] = Pixel::from_raw_slice_mut(data.as_mut_slice());

                    for pix in color_buffer.iter_mut() {
                        // See
                        // https://github.com/erisir/FIJI/blob/a30ce62566b7a441bc315c8fff365b9985779b27/src-plugins/Color_Inspector_3D/src/main/java/Color_Inspector_3D.java#L4391-L4472

                        let rgb: palette::rgb::Rgb<_, u8> = pix.color;
                        let rgb_f32: palette::rgb::Rgb<_, f32> = rgb.into_format();

                        use palette::ConvertInto;
                        let mut hsl_f32: palette::Hsl<palette::encoding::Srgb, f32> =
                            rgb_f32.convert_into();

                        hsl_f32.hue =
                            palette::RgbHue::from_degrees(hsl_f32.hue.to_degrees() + 180.0);
                        // hsl_f32.clamp_self();

                        hsl_f32.saturation *= 4.0;
                        // hsl_f32.clamp_self();

                        let rgb_f32: palette::rgb::Rgb<_, f32> = hsl_f32.convert_into();
                        let rgb_u8: palette::rgb::Rgb<_, u8> = rgb_f32.into_format();
                        pix.color = rgb_u8;
                    }

                    web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                        Clamped(data.as_mut_slice()),
                        w,
                        h,
                    )
                    .unwrap()
                };
                ctx2.put_image_data(&new_data, 0.0, 0.0).unwrap();
            }
        }
    }
}
