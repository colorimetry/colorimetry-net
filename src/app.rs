// See https://github.com/AvraamMavridis/wasm-image-to-black-white

use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, Node, Url};
use yew::prelude::*;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::virtual_dom::VNode;
use yew::{html, ChangeData, Component, ComponentLink, Html, NodeRef, ShouldRender};

pub struct App {
    link: ComponentLink<Self>,
    image_loaded_closure: Closure<dyn FnMut(JsValue)>,
    tasks: Vec<ReaderTask>,
    file_info: Option<FileInfo>,
}

struct FileInfo {
    file_data: FileData,
    img: HtmlImageElement,
    node_ref: NodeRef,
}

pub enum Msg {
    FileLoaded(FileData),
    Files(Vec<File>),
    ImageLoaded,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link2 = link.clone();
        let image_loaded_closure = Closure::wrap(Box::new(move |_| {
            link2.send_message(Msg::ImageLoaded);
        }) as Box<dyn FnMut(JsValue)>);

        App {
            link,
            image_loaded_closure,
            tasks: vec![],
            file_info: None,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ImageLoaded => {
                // We have this just for the side effect of rendering again.
            }
            Msg::FileLoaded(file_data) => {
                let buffer = Uint8Array::from(file_data.content.as_slice());
                let buffer_val: &JsValue = buffer.as_ref();
                let parts = Array::new_with_length(1);
                parts.set(0, buffer_val.clone());
                let blob = Blob::new_with_u8_array_sequence(parts.as_ref()).unwrap();
                let img = HtmlImageElement::new().unwrap();

                img.set_onload(Some(self.image_loaded_closure.as_ref().unchecked_ref()));

                img.set_src(&Url::create_object_url_with_blob(&blob).unwrap());

                let node_ref = NodeRef::default();

                self.file_info = Some(FileInfo {
                    file_data,
                    node_ref,
                    img,
                });
            }
            Msg::Files(files) => {
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
        html! {
            <div class="colorswitch-wrapper">
                <section class="main">
                    <div>
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

                    { self.view_file() }

                </section>

                <footer class="info">
                    <p>{ "Source code " }<a href="https://github.com/strawlab/colorswitch">{ "github.com/strawlab/colorswitch" }</a></p>
                </footer>
            </div>
        }
    }
}

impl App {
    fn view_file(&self) -> Html {
        if let Some(file_info) = self.file_info.as_ref() {
            log::info!("view 1");
            if let Some(canvas) = file_info.node_ref.cast::<HtmlCanvasElement>() {
                log::info!("view 2");

                let ctx = CanvasRenderingContext2d::from(JsValue::from(
                    canvas.get_context("2d").unwrap().unwrap(),
                ));

                log::info!(
                    "{}: {}x{}",
                    file_info.file_data.name,
                    file_info.img.width(),
                    file_info.img.height()
                );

                ctx.draw_image_with_html_image_element(&file_info.img, 0.0, 0.0)
                    .unwrap();
            }

            html! {
                <canvas ref={file_info.node_ref.clone()} />
            }
        } else {
            html! {}
        }
    }
}
