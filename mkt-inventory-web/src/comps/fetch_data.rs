use mkt_data::MktData;
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::agents::data::{DataRequest, DataStore};

pub enum Msg {
    DataStore(ReadOnly<DataStore>),
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

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataStore);
        let mut data_store = DataStore::bridge(callback);
        data_store.send(DataRequest::Load);
        Self { data_store }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStore(_) => {}
            Msg::Fetch => {
                ctx.link().send_future(async {
                    Msg::DoneFetching(Box::new(DataStore::load_data().await))
                });
            }
            Msg::DoneFetching(data) => {
                self.data_store.send(DataRequest::New(data));
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
