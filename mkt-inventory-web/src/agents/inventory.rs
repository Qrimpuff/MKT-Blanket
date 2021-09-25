use gloo::storage::{LocalStorage, Storage};
use mkt_data::MktInventory;
use yew_agent::{
    utils::store::{Store, StoreWrapper},
    AgentLink,
};

pub enum Msg {
    Replace(Box<MktInventory>),
    Merge(Box<MktInventory>),
    Refresh,
}

pub enum InventoryRequest {
    Add(Box<MktInventory>),
    Load,
    Save,
    Refresh,
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
            InventoryRequest::Add(inv) => {
                link.send_message(Msg::Merge(inv));
                link.send_input(InventoryRequest::Save);
            }
            InventoryRequest::Load => {
                if let Ok(inv) = LocalStorage::get("mkt_inventory") {
                    link.send_message(Msg::Replace(inv));
                }
            }
            InventoryRequest::Save => {
                LocalStorage::set("mkt_inventory", &self.inv).unwrap();
            }
            InventoryRequest::Refresh => link.send_message(Msg::Refresh),
        }
    }

    fn reduce(&mut self, msg: Self::Action) {
        match msg {
            Msg::Replace(inv) => {
                self.inv = *inv;
            }
            Msg::Merge(inv) => {
                self.inv.update_inventory(*inv);
            }
            Msg::Refresh => { /* do nothing */ }
        }
    }
}
