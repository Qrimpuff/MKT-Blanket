use mkt_data::MktData;
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, StoreWrapper},
    Bridge,
};

use crate::agents::data::{DataRequest, DataStore};

pub enum Msg {
    Fetch,
    DoneFetching(Box<MktData>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct FetchData {
    data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
}

impl Component for FetchData {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            data_store: DataStore::bridge(Callback::noop()),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Fetch => {
                ctx.link().send_future(async {
                    Msg::DoneFetching(Box::new(DataStore::load_data().await))
                });
            }
            Msg::DoneFetching(data) => {
                self.data_store.send(DataRequest::NewData(data));
            }
        };
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <button onclick={ctx.link().callback(|_| Msg::Fetch)}>
                    { "fetch data" }
                </button>
            </>
        }
    }
}
