use gloo::storage::{LocalStorage, Storage};
use mkt_data::MktInventory;
use yew::prelude::*;

use super::data_manager::download_file;

#[derive(Clone)]
pub enum Msg {
    Download,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct DownloadInv {}

impl Component for DownloadInv {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Download => {
                if let Ok(inv) = LocalStorage::get("mkt_inventory") {
                    let inv: MktInventory = inv;
                    let json = serde_json::to_string_pretty(&inv).unwrap();
                    download_file("mkt_inventory.json", &json);
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <button class={classes!("button", "is-info")} onclick={ctx.link().callback(|_| Msg::Download)}>
                    <span>{ "Download Inventory" }</span>
                    <span class="icon"><i class="fas fa-download"/></span>
                </button>
            </>
        }
    }
}
