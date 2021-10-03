use gloo::storage::{LocalStorage, Storage};
use mkt_data::MktData;
use yew::prelude::*;

use super::data_manager::download_file;

#[derive(Clone)]
pub enum Msg {
    Download,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct DownloadHash {}

impl Component for DownloadHash {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Download => {
                if let Ok(data) = LocalStorage::get("mkt_data") {
                    let data: MktData = data;
                    let mut data_hash = data.hashes();
                    if let Ok(hash) = LocalStorage::get("mkt_hash") {
                        data_hash.merge(hash);
                    };
                    let json = serde_json::to_string_pretty(&data_hash).unwrap();
                    download_file("mkt_hash.json", json.as_str());
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <button class={classes!("button", "is-info")} onclick={ctx.link().callback(|_| Msg::Download)}>
                    <span>{ "Download Hashes" }</span>
                    <span class="icon"><i class="fas fa-download"/></span>
                </button>
            </>
        }
    }
}
