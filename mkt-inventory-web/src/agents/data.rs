use mkt_data::MktData;
use reqwest::Url;
use yew::utils;
use yew_agent::{
    utils::store::{Store, StoreWrapper},
    AgentLink,
};

pub enum Msg {
    NewData(Box<MktData>),
}

pub enum DataRequest {
    NewData(Box<MktData>),
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
            DataRequest::NewData(data) => {
                link.send_message(Msg::NewData(data));
            }
        }
    }

    fn reduce(&mut self, msg: Self::Action) {
        match msg {
            Msg::NewData(data) => {
                self.data = *data;
            }
        }
    }
}

impl DataStore {
    pub async fn load_data() -> MktData {
        let base = Url::parse(&utils::origin().unwrap()).unwrap();
        let url = base.join("mkt_data.json").unwrap();
        let resp = reqwest::get(url).await.unwrap();
        let json = resp.text().await.unwrap();
        MktData::from_json(&json).unwrap()
    }
}
