use mkt_data::{ItemId, ItemType};
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::{agents::data::DataStore, comps::item::Item};

pub enum Msg {
    DataStore(ReadOnly<DataStore>),
    Toggle,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub i_type: ItemType,
}

pub struct ItemList {
    item_ids: Vec<ItemId>,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    visible: bool,
}

impl Component for ItemList {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataStore);
        Self {
            item_ids: Vec::new(),
            _data_store: DataStore::bridge(callback),
            visible: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStore(state) => {
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
            Msg::Toggle => {
                self.visible = !self.visible;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let title = match ctx.props().i_type {
            ItemType::Driver => "Drivers",
            ItemType::Kart => "Karts",
            ItemType::Glider => "Gliders",
        };
        let items = if self.visible {
            html! {
                <ul>
                { for self.item_ids.iter().map(|id| html!{
                    <li><Item id={id.clone()} /></li>
                }) }
                </ul>
            }
        } else {
            html! {}
        };
        html! {
            <>
                <h2 class="subtitle">
                    { title }{" "}
                    <button onclick={ctx.link().callback(|_| Msg::Toggle)}>{ if self.visible {'-'} else {'+'} }</button>
                </h2>
                { items }
            </>
        }
    }
}
