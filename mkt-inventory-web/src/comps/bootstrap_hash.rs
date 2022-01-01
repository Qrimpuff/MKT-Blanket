use gloo::file::{self, callbacks::FileReader, File};
use mkt_data::ItemType;
use mkt_import::screenshot::BootstrapError;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::agents::import::{ImportAgent, ImportRequest, ImportResponse};

pub enum Msg {
    Up(usize),
    Down(usize),
    Remove(usize),
    Files(Vec<File>),
    Loaded(usize, Vec<u8>),
    TypeChanged(ItemType),
    Bootstrap,
    ImportResponse(ImportResponse),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct BootstrapHash {
    working: bool,
    error: String,
    i_type: ItemType,
    readers: Vec<(String, FileReader, Vec<u8>)>,
    import: Box<dyn Bridge<ImportAgent>>,
}

impl Component for BootstrapHash {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::ImportResponse);
        Self {
            working: false,
            error: "".into(),
            i_type: ItemType::Driver,
            readers: vec![],
            import: ImportAgent::bridge(callback),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Files(mut files) => {
                files.sort_by_key(|f| f.name());
                for file in files.into_iter() {
                    let i = self.readers.len();
                    let task = {
                        let link = ctx.link().clone();

                        file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded(i, res.unwrap()))
                        })
                    };
                    self.readers.push((file.name(), task, vec![]));
                }
                true
            }
            Msg::Loaded(i, bytes) => {
                self.readers[i].2 = bytes;
                false
            }
            Msg::Up(i) => {
                if i as i32 > 0 {
                    self.readers.swap(i, i - 1);
                    true
                } else {
                    false
                }
            }
            Msg::Down(i) => {
                if i + 1 < self.readers.len() {
                    self.readers.swap(i, i + 1);
                    true
                } else {
                    false
                }
            }
            Msg::Remove(i) => {
                self.readers.remove(i);
                true
            }
            Msg::TypeChanged(i_type) => {
                self.i_type = i_type;
                true
            }
            Msg::Bootstrap => {
                self.working = true;
                let bytes = self.readers.iter().map(|f| f.2.clone()).collect();
                self.import
                    .send(ImportRequest::BootstrapItemHashes(self.i_type, bytes));
                true
            }
            Msg::ImportResponse(resp) => {
                self.working = false;
                match resp {
                    ImportResponse::BootstrapError(error) => match error {
                        BootstrapError::WrongLength(size, expected) => {
                            self.error = format!(
                                "There were {} items, but {} were expected.",
                                size, expected
                            );
                        }
                        BootstrapError::MissingId => {
                            self.error = "Missing Id".into();
                        }
                    },
                    ImportResponse::BootstrapSuccess => {
                        self.error = "".into();
                        self.readers = vec![];
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            <h2 class="subtitle is-4">{"Bootstrap Hashes"}</h2>
            <div class="block">
                <p class="block">{ "Choose screenshots to bootstrap" }</p>
                {
                    if !self.error.is_empty() {
                        html! {
                            <article class="message is-danger">
                                <div class="message-body">
                                    { &self.error }
                                </div>
                            </article>
                        }
                    } else {
                        html! {}
                    }
                }
                <div class="buttons has-addons">
                    { self.view_type_button(ctx, "Drivers", ItemType::Driver) }
                    { self.view_type_button(ctx, "Karts", ItemType::Kart) }
                    { self.view_type_button(ctx, "Gliders", ItemType::Glider) }
                </div>
                <div class="buttons">
                    <button class={classes!("button", "is-success")} onclick={ctx.link().callback(|_| Msg::Bootstrap)}>
                        <span class="icon"><i class="fas fa-upload"/></span>
                        <span>{ "Bootstrap" }</span>
                    </button>
                </div>
                { if self.working {
                    html! {<progress class="progress" max={1} />}
                } else {
                    html! {}
                }}
                <div class="file mb-3">
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
                            input.set_files(None);
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
                <ul>
                { for self.readers.iter().enumerate().map(|(i, f)| {
                    html!{
                        <>
                        <li class="mb-2">
                            <div class="buttons has-addons">
                                <button class="button" disabled={i==0} onclick={ctx.link().callback(move |_| Msg::Up(i))}>
                                    <span class="icon">
                                        <i class="fas fa-arrow-up"></i>
                                    </span>
                                </button>
                                <button class="button" disabled={i==self.readers.len()-1} onclick={ctx.link().callback(move |_| Msg::Down(i))}>
                                    <span class="icon">
                                        <i class="fas fa-arrow-down"></i>
                                    </span>
                                </button>
                                <span class="button is-static">{ &f.0 }</span>
                                <button class="button is-danger" onclick={ctx.link().callback(move |_| Msg::Remove(i))}>
                                    <span class="icon">
                                        <i class="fas fa-times"></i>
                                    </span>
                                </button>
                            </div>
                        </li>
                        </>
                    }
                }) }
                </ul>
            </div>
            </>
        }
    }
}
impl BootstrapHash {
    fn view_type_button(&self, ctx: &Context<Self>, text: &str, i_type: ItemType) -> Html {
        html! {
            <button
            class={classes!("button", (self.i_type == i_type).then_some("is-info is-selected"))}
            onclick={ctx.link().callback(move |_| Msg::TypeChanged(i_type))}>
            <span>{ text }</span>
            </button>
        }
    }
}
