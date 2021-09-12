use gloo::storage::{LocalStorage, Storage};
use mkt_data::MktInventory;
use yew_agent::{
    utils::store::{Store, StoreWrapper},
    AgentLink,
};

pub enum Msg {
    Inventory(Box<MktInventory>),
}

pub enum InventoryRequest {
    New(Box<MktInventory>),
    Load,
    Save,
}

pub struct Inventory {
    pub inv: MktInventory,
}

impl Store for Inventory {
    type Action = Msg;
    type Input = InventoryRequest;

    fn new() -> Self {
        let inv = MktInventory::new();
        Self { inv }
    }

    fn handle_input(&self, link: AgentLink<StoreWrapper<Self>>, msg: Self::Input) {
        match msg {
            InventoryRequest::New(inv) => {
                link.send_message(Msg::Inventory(inv));
                link.send_input(InventoryRequest::Save);
            }
            InventoryRequest::Load => {
                if let Ok(inv) = LocalStorage::get("mkt_inventory") {
                    link.send_message(Msg::Inventory(inv));
                }
            }
            InventoryRequest::Save => {
                LocalStorage::set("mkt_inventory", &self.inv).unwrap();
            }
        }
    }

    fn reduce(&mut self, msg: Self::Action) {
        match msg {
            Msg::Inventory(inv) => {
                self.inv = *inv;
            }
        }
    }
}
