use gloo::file::{self, callbacks::FileReader, File};
use mkt_import::*;
use yew::{
    prelude::*,
    web_sys::{self, HtmlInputElement},
};
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::agents::{
    data::DataStore,
    inventory::{Inventory, InventoryRequest},
};

pub enum Msg {
    DataStore(ReadOnly<DataStore>),
    Inventory(ReadOnly<Inventory>),
    Files(Vec<File>),
    Loaded(String, Vec<u8>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct Import {
    readers: Vec<FileReader>,
    data: Option<ReadOnly<DataStore>>,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Component for Import {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::Inventory);
        let mut inventory = Inventory::bridge(callback);
        inventory.send(InventoryRequest::Load);

        let callback = ctx.link().callback(Msg::DataStore);

        Self {
            readers: vec![],
            inventory,
            _data_store: DataStore::bridge(callback),
            data: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStore(state) => {
                self.data = Some(state);
            }
            Msg::Inventory(_) => {}

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
            }
            Msg::Loaded(_file_name, bytes) => {
                let (inv, _miss) = screenshot::image_bytes_to_inventory(
                    bytes,
                    &self.data.as_ref().unwrap().borrow().data,
                );
                self.inventory.send(InventoryRequest::Add(Box::from(inv)));
            }
        };
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <button>
                    { "import inventory" }
                </button>
                <p>{ "Choose a file to upload to see the uploaded bytes" }</p>
                <input type="file" multiple=true onchange={ctx.link().callback(move |e: Event| {
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
                    })}
                />
            </>
        }
    }
}
