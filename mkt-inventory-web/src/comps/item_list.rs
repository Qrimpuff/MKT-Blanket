use mkt_data::{ItemId, ItemType};
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::{agents::data::DataStore, comps::item::Item};

pub enum Msg {
    DataStoreMsg(ReadOnly<DataStore>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub i_type: ItemType,
}

pub struct ItemList {
    item_ids: Vec<ItemId>,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
}

impl Component for ItemList {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataStoreMsg);
        Self {
            item_ids: Vec::new(),
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
                if items.len() != self.item_ids.len() {
                    self.item_ids = items.keys().cloned().collect();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let title = match ctx.props().i_type {
            ItemType::Driver => "Drivers",
            ItemType::Kart => "Karts",
            ItemType::Glider => "Gliders",
        };
        html! {
            <>
                <h2>{ title }</h2>
                <ul>
                { for self.item_ids.iter().map(|id| html!{
                    <li><Item i_type={ctx.props().i_type} id={id.clone()} /></li>
                }) }
                </ul>
            </>
        }
    }
}
