use gloo::storage::{LocalStorage, Storage};
use mkt_data::{MktInventory, MktItemHashes};
use mkt_import::screenshot;
use serde::{Deserialize, Serialize};
use yew::Callback;
use yew_agent::{
    utils::store::{Bridgeable, StoreWrapper},
    Agent, AgentLink, Bridge, Job,
};

use super::inventory::{Inventory, InventoryRequest};

pub enum Msg {
    UpdateInventory(MktInventory),
    UpdateHashes(MktItemHashes),
}

#[derive(Serialize, Deserialize)]
pub enum ImportRequest {
    ImportScreenshot(Vec<u8>),
    BootstrapItemHashes(Vec<Vec<u8>>),
}

pub struct ImportAgent {
    pub link: AgentLink<Self>,
    pub inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Agent for ImportAgent {
    type Reach = Job<Self>;
    type Message = Msg;
    type Input = ImportRequest;
    type Output = ();

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            inventory: Inventory::bridge(Callback::noop()),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        // update state
        match msg {
            Msg::UpdateInventory(inv) => {
                self.inventory.send(InventoryRequest::Add(Box::from(inv)));
            }
            Msg::UpdateHashes(new_hash) => {
                let hash: Option<MktItemHashes> = LocalStorage::get("mkt_hash").ok();
                // update local hashes
                if !new_hash.hashes.is_empty() {
                    let mut hash = hash.unwrap_or_default();
                    hash.merge(new_hash);
                    LocalStorage::set("mkt_hash", hash).unwrap();
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _id: yew_agent::HandlerId) {
        match msg {
            ImportRequest::ImportScreenshot(bytes) => {
                let hash = LocalStorage::get("mkt_hash").ok();
                let data = LocalStorage::get("mkt_data").ok();

                let (inv, new_hash) = screenshot::image_bytes_to_inventory(
                    bytes,
                    data.as_ref().unwrap(),
                    hash.as_ref(),
                );

                self.link.send_message(Msg::UpdateInventory(inv));
                self.link.send_message(Msg::UpdateHashes(new_hash));
            }
            ImportRequest::BootstrapItemHashes(_imgs) => todo!(),
        }
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}
