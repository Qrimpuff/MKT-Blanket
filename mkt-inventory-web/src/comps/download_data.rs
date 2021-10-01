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

pub struct DownloadData {}

impl Component for DownloadData {
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
                    let json = serde_json::to_string_pretty(&data).unwrap();
                    download_file("mkt_data.json", &json);
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <button class={classes!("button", "is-info")} onclick={ctx.link().callback(|_| Msg::Download)}>
                    <span>{ "Download Data" }</span>
                    <span class="icon"><i class="fas fa-download"/></span>
                </button>
            </>
        }
    }
}
