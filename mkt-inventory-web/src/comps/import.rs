use chrono::{DateTime, Utc};
use gloo::{
    file::{self, callbacks::FileReader, File},
    timers::callback::Timeout,
};
use itertools::Itertools;
use mkt_data::ItemType;
use web_sys::HtmlInputElement;
use yew::{
    prelude::*,
};
use yew_agent::{Bridge, Bridged};

use crate::{
    agents::{
        data_inventory::{DataInvItem, DataInventory, DataInventoryAgent, Shared},
        import::{ImportAgent, ImportRequest},
    },
    comps::item::ShowStat,
};

use super::item::Item;

pub enum Msg {
    Files(Vec<File>),
    Loaded(String, Vec<u8>),
    Done,
    DataInventory(Shared<DataInventory>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct Import {
    readers: Vec<FileReader>,
    completed: usize,
    timeout: Option<Timeout>,
    last_changed: DateTime<Utc>,
    modified_items: Vec<Shared<DataInvItem>>,
    import: Box<dyn Bridge<ImportAgent>>,
    _data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for Import {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataInventory);
        Self {
            readers: vec![],
            completed: 0,
            timeout: None,
            last_changed: Utc::now(),
            modified_items: Vec::new(),
            import: ImportAgent::bridge(Callback::noop()),
            _data_inventory: DataInventoryAgent::bridge(callback),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Files(files) => {
                self.modified_items = vec![];
                self.last_changed = Utc::now();
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
            Msg::DataInventory(state) => {
                let state = state.read().unwrap();
                self.modified_items = state
                    .drivers
                    .values()
                    .chain(state.karts.values())
                    .chain(state.gliders.values())
                    .filter(|i| {
                        Some(self.last_changed)
                            <= i.read().unwrap().inv.as_ref().and_then(|i| i.last_changed)
                    })
                    .cloned()
                    .collect();
                self.modified_items
                    .sort_by_key(|i| i.read().unwrap().data.id.clone());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let drivers = self
            .modified_items
            .iter()
            .filter(|r| r.read().unwrap().data.i_type == ItemType::Driver)
            .map(|r| r.read().unwrap().data.id.clone())
            .collect_vec();
        let karts = self
            .modified_items
            .iter()
            .filter(|r| r.read().unwrap().data.i_type == ItemType::Kart)
            .map(|r| r.read().unwrap().data.id.clone())
            .collect_vec();
        let gliders = self
            .modified_items
            .iter()
            .filter(|r| r.read().unwrap().data.i_type == ItemType::Glider)
            .map(|r| r.read().unwrap().data.id.clone())
            .collect_vec();
        let items = html! {
            <>
            <p class="subtitle is-6">{"Drivers "}<b>{drivers.len()}</b></p>
            <div class="columns is-multiline">
            { for drivers.iter().map(|id| html!{ <div class="column is-one-quarter py-1"><Item id={id.clone()} show_stat={ShowStat::LevelPoints} /></div> }) }
            </div>
            <p class="subtitle is-6">{"Karts "}<b>{karts.len()}</b></p>
            <div class="columns is-multiline">
            { for karts.iter().map(|id| html!{ <div class="column is-one-quarter py-1"><Item id={id.clone()} show_stat={ShowStat::LevelPoints} /></div> }) }
            </div>
            <p class="subtitle is-6">{"Gliders "}<b>{gliders.len()}</b></p>
            <div class="columns is-multiline">
            { for gliders.iter().map(|id| html!{ <div class="column is-one-quarter py-1"><Item id={id.clone()} show_stat={ShowStat::LevelPoints} /></div> }) }
            </div>
            </>
        };
        html! {
            <>
            <h2 class="title is-4">{"Import / Export"}</h2>
            <div class="block">
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
                { if !self.readers.is_empty() {
                    html! {<progress class="progress" value={Some(self.completed).filter(|c| *c > 0).map(|c| c.to_string())} max={self.readers.len().to_string()} />}
                } else {
                    html! {}
                }}
            </div>
            <h3 class="subtitle is-4">{"Modified Items "}<b>{self.modified_items.len()}</b></h3>
            <div>{ items }</div>
            </>
        }
    }
}
