use gloo::storage::{LocalStorage, Storage};
use mkt_data::MktData;
use yew_agent::{
    utils::store::{Store, StoreWrapper},
    AgentLink,
};

pub enum Msg {
    Data(Box<MktData>),
}

pub enum DataRequest {
    New(Box<MktData>),
    Load,
    Save,
    Delete,
}

pub struct DataStore {
    pub data: MktData,
}

impl Store for DataStore {
    type Action = Msg;
    type Input = DataRequest;

    fn new() -> Self {
        let data = MktData::new();
        Self { data }
    }

    fn handle_input(&self, link: AgentLink<StoreWrapper<Self>>, msg: Self::Input) {
        match msg {
            DataRequest::New(data) => {
                link.send_message(Msg::Data(data));
                link.send_input(DataRequest::Save);
            }
            DataRequest::Load => {
                if let Ok(data) = LocalStorage::get("mkt_data") {
                    link.send_message(Msg::Data(data));
                }
            }
            DataRequest::Save => {
                LocalStorage::set("mkt_data", &self.data).unwrap();
            }
            DataRequest::Delete => {
                link.send_input(DataRequest::New(Box::new(MktData::new())));
            }
        }
    }

    fn reduce(&mut self, msg: Self::Action) {
        match msg {
            Msg::Data(data) => {
                self.data = *data;
            }
        }
    }
}
