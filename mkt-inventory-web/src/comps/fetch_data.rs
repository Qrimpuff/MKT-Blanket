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
    fetching: bool,
    data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
}

impl Component for FetchData {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataStore);
        let mut data_store = DataStore::bridge(callback);
        data_store.send(DataRequest::Load);
        Self { fetching: false, data_store }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStore(_) => false,
            Msg::Fetch => {
                self.fetching = true;
                ctx.link().send_future(async {
                    Msg::DoneFetching(Box::new(DataStore::load_data().await))
                });
                true
            }
            Msg::DoneFetching(data) => {
                self.fetching = false;
                self.data_store.send(DataRequest::New(data));
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <button class={classes!("button", "is-info", self.fetching.then_some("is-loading"))} onclick={ctx.link().callback(|_| Msg::Fetch)}>
                    { "fetch data" }
                </button>
            </>
        }
    }
}
