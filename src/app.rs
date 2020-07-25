// See https://github.com/AvraamMavridis/wasm-image-to-black-white

use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsValue;
use web_sys::{Blob, HtmlImageElement, Node, Url};
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::virtual_dom::VNode;
use yew::{html, ChangeData, Component, ComponentLink, Html, ShouldRender};

pub struct App {
    link: ComponentLink<Self>,
    tasks: Vec<ReaderTask>,
    files: Vec<FileInfo>,
}

struct FileInfo {
    _file_data: FileData,
    img: HtmlImageElement,
}

pub enum Msg {
    Loaded(FileData),
    Files(Vec<File>),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link,
            tasks: vec![],
            files: vec![],
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Loaded(file_data) => {
                let buffer = Uint8Array::from(file_data.content.as_slice());
                let buffer_val: &JsValue = buffer.as_ref();
                let parts = Array::new_with_length(1);
                parts.set(0, buffer_val.clone());
                let blob = Blob::new_with_u8_array_sequence(parts.as_ref()).unwrap();
                let img = HtmlImageElement::new().unwrap();
                img.set_src(&Url::create_object_url_with_blob(&blob).unwrap());

                self.files.push(FileInfo {
                    _file_data: file_data,
                    img,
                });
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let task = {
                        let callback = self.link.callback(Msg::Loaded);
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

                    <ul class="file-list">
                        { for self.files.iter().enumerate().map(|e| self.view_file(e)) }
                    </ul>
                </section>

                <footer class="info">
                    <p>{ "Source code " }<a href="https://github.com/strawlab/colorswitch">{ "github.com/strawlab/colorswitch" }</a></p>
                </footer>
            </div>
        }
    }
}

impl App {
    fn view_file(&self, (idx, file_info): (usize, &FileInfo)) -> Html {
        // https://github.com/PsichiX/Oxygengine/blob/208b9d76c3bb6d2b29e320656dfaa0c8b30397ce/oxygengine-composite-renderer-backend-web/src/lib.rs
        let node = Node::from(file_info.img.clone());
        let vnode = VNode::VRef(node);
        vnode
    }
}
