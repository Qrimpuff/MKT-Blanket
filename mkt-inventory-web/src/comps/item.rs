use mkt_data::{ItemId, ItemType};
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::agents::data::DataStore;

pub enum Msg {
    DataStoreMsg(ReadOnly<DataStore>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub i_type: ItemType,
    pub id: ItemId,
}

pub struct Item {
    item: Option<mkt_data::Item>,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
}

impl Component for Item {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataStoreMsg);
        Self {
            item: None,
            _data_store: DataStore::bridge(callback),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStoreMsg(state) => {
                let state = state.borrow();

                let items = match ctx.props().i_type {
                    ItemType::Driver => &state.data.drivers,
                    ItemType::Kart => &state.data.karts,
                    ItemType::Glider => &state.data.gliders,
                };
                let item = items.get(&ctx.props().id);

                if item != self.item.as_ref() {
                    self.item = item.cloned();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if let Some(item) = self.item.as_ref() {
            html! {
                <div>{ &item.name }</div>
            }
        } else {
            html! {
                <div>{ "no_item" }</div>
            }
        }
    }
}
