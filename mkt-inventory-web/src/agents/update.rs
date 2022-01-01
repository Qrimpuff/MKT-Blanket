use chrono::{DateTime, Datelike, Utc};
use gloo::storage::{LocalStorage, Storage};
use mkt_data::MktData;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use yew::Callback;
use yew_agent::{
    utils::store::{Bridgeable, StoreWrapper},
    Agent, AgentLink, Bridge, HandlerId, Job,
};

use super::data::{DataRequest, DataStore};

pub enum Msg {
    UpdateData(HandlerId, Option<Box<MktData>>),
}

#[derive(Serialize, Deserialize)]
pub enum UpdateRequest {
    CheckUpdateData,
}

pub enum UpdateResponse {
    NoDataUpdate,
    DoneDataUpdate,
}

pub struct UpdateAgent {
    pub link: AgentLink<Self>,
    data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
}

impl Agent for UpdateAgent {
    type Reach = Job<Self>;
    type Message = Msg;
    type Input = UpdateRequest;
    type Output = UpdateResponse;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            data_store: DataStore::bridge(Callback::noop()),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::UpdateData(id, data) => {
                if let Some(data) = data {
                    self.data_store.send(DataRequest::New(data));
                    self.link.respond(id, UpdateResponse::DoneDataUpdate)
                } else {
                    self.link.respond(id, UpdateResponse::NoDataUpdate)
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            UpdateRequest::CheckUpdateData => {
                self.link.send_future(async move {
                    let date = UpdateAgent::get_last_modified_date("mkt_data.json")
                        .await
                        .unwrap_or_else(Utc::now);
                    if date
                        > LocalStorage::get("mkt_data_date")
                            .unwrap_or_else(|_| Utc::now().with_year(2000).unwrap())
                    {
                        if let Some(data) = UpdateAgent::load_data().await {
                            LocalStorage::set("mkt_data_date", &date).unwrap();
                            return Msg::UpdateData(id, Some(Box::new(data)));
                        }
                    }
                    Msg::UpdateData(id, None)
                });
            }
        }
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}

impl UpdateAgent {
    pub async fn load_data() -> Option<MktData> {
        let base = Url::parse(&gloo_utils::window().origin()).ok()?;
        let mut url = base.join("MKT-Blanket/mkt_data.json").ok()?;
        url.set_query(Some(&format!("day={}", Utc::today())));
        let resp = reqwest::get(url).await.ok()?;
        let json = resp.text().await.ok()?;
        MktData::from_json(&json).ok()
    }

    pub async fn get_last_modified_date(file: &str) -> Option<DateTime<Utc>> {
        // for testing
        if gloo_utils::window().origin().contains("localhost") {
            return Some(Utc::now());
        }

        let resp = reqwest::get(format!("https://api.github.com/repos/qrimpuff/MKT-Blanket/commits?path={}&per_page=1&sha=gh-pages", file)).await.ok()?;
        let content = resp.text().await.ok()?;

        #[derive(Deserialize, Debug)]
        struct A {
            commit: B,
        }
        #[derive(Deserialize, Debug)]
        struct B {
            committer: C,
        }
        #[derive(Deserialize, Debug)]
        struct C {
            date: DateTime<Utc>,
        }

        //commit.committer.date
        let r: Vec<A> = serde_json::from_str(&content).unwrap();

        Some(r[0].commit.committer.date)
    }
}
