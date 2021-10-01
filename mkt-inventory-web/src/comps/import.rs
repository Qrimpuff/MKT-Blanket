use gloo::{
    file::{self, callbacks::FileReader, File},
    timers::callback::Timeout,
};
use yew::{
    prelude::*,
    web_sys::{self, HtmlInputElement},
};
use yew_agent::{Bridge, Bridged};

use crate::agents::import::{ImportAgent, ImportRequest};

pub enum Msg {
    Files(Vec<File>),
    Loaded(String, Vec<u8>),
    Done,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct Import {
    readers: Vec<FileReader>,
    completed: usize,
    timeout: Option<Timeout>,
    import: Box<dyn Bridge<ImportAgent>>,
}

impl Component for Import {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: vec![],
            completed: 0,
            timeout: None,
            import: ImportAgent::bridge(Callback::noop()),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let task = {
                        let file_name = file.name();
                        let link = ctx.link().clone();

                        file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded(file_name, res.unwrap()))
                        })
                    };
                    self.readers.push(task);
                }
                true
            }
            Msg::Loaded(_file_name, bytes) => {
                self.import.send(ImportRequest::ImportScreenshot(bytes));

                self.completed += 1;
                if self.completed == self.readers.len() {
                    let handle = {
                        let link = ctx.link().clone();
                        Timeout::new(1_000, move || link.send_message(Msg::Done))
                    };
                    self.timeout = Some(handle);
                }
                true
            }
            Msg::Done => {
                self.readers = vec![];
                self.completed = 0;
                self.timeout = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="block">
                <h2 class="subtitle">{ "Import Inventory" }</h2>
                <p>{ "Choose a screenshot to import" }</p>
                <div class="file">
                <label class="file-label">
                    <input class="file-input" type="file" accept=".jpg,image/jpeg,.png,image/png" multiple=true onchange={ctx.link().callback(move |e: Event| {
                        let mut result = Vec::new();
                        let input: HtmlInputElement = e.target_unchecked_into();

                        if let Some(files) = input.files() {
                            let files = js_sys::try_iter(&files)
                                .unwrap()
                                .unwrap()
                                .map(|v| web_sys::File::from(v.unwrap()))
                                .map(File::from);
                            result.extend(files);
                        }
                        Msg::Files(result)
                    })} />
                    <span class="file-cta">
                        <span class="file-icon">
                            <i class="fas fa-upload"></i>
                        </span>
                        <span class="file-label">
                            { "Choose a file…" }
                        </span>
                    </span>
                </label>
                </div>
                { if !self.readers.is_empty() {
                    html! {<progress class="progress" value={Some(self.completed).filter(|c| *c > 0).map(|c| c.to_string())} max={self.readers.len().to_string()} />}
                } else {
                    html! {}
                }}
            </div>
        }
    }
}
