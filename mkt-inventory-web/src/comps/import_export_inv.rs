use gloo::{
    file::{self, callbacks::FileReader, File},
    storage::{LocalStorage, Storage},
    timers::callback::Timeout,
};
use mkt_data::MktInventory;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, StoreWrapper},
    Bridge,
};

use crate::agents::inventory::{Inventory, InventoryRequest};

use super::data_manager::download_file;

pub enum Msg {
    Files(Vec<File>),
    Loaded(String, String),
    Done,
    Download,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct ImportExportInv {
    readers: Vec<FileReader>,
    completed: usize,
    timeout: Option<Timeout>,
    pub inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Component for ImportExportInv {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: vec![],
            completed: 0,
            timeout: None,
            inventory: Inventory::bridge(Callback::noop()),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let task = {
                        let file_name = file.name();
                        let link = ctx.link().clone();

                        file::callbacks::read_as_text(&file, move |res| {
                            link.send_message(Msg::Loaded(file_name, res.unwrap()))
                        })
                    };
                    self.readers.push(task);
                }
                true
            }
            Msg::Loaded(_file_name, json) => {
                if let Ok(inv) = MktInventory::from_json(&json) {
                    self.inventory.send(InventoryRequest::Add(Box::from(inv)));
                }

                self.completed += 1;
                if self.completed == self.readers.len() {
                    let handle = {
                        let link = ctx.link().clone();
                        Timeout::new(3_000, move || link.send_message(Msg::Done))
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
            Msg::Download => {
                if let Ok(inv) = LocalStorage::get("mkt_inventory") {
                    let inv: MktInventory = inv;
                    let json = serde_json::to_string_pretty(&inv).unwrap();
                    download_file("mkt_inventory.json", json.as_str());
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="block">
            <div class="field is-grouped">
                <p class="control">
                    <button class={classes!("button", "is-info")} onclick={ctx.link().callback(|_| Msg::Download)}>
                        <span class="icon"><i class="fas fa-download"/></span>
                        <span>{ "Download Inventory" }</span>
                    </button>
                </p>

                <p class="control">
                    <div class="file">
                    <label class="file-label">
                        <input class="file-input" type="file" accept=".json" onchange={ctx.link().callback(move |e: Event| {
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
                                { "Choose an inventory file…" }
                            </span>
                        </span>
                    </label>
                    </div>
                </p>

                { if self.timeout.is_some() {
                    html! {
                        <p class="control">{ "Inventory loaded!" }</p>
                    }
                } else {
                    html! {}
                }}
            </div>
            </div>
        }
    }
}
