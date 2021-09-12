use mkt_data::{item_type_from_id, ItemId, ItemType};
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::agents::{data::DataStore, inventory::Inventory};

pub enum Msg {
    DataStore(ReadOnly<DataStore>),
    Inventory(ReadOnly<Inventory>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: ItemId,
}

pub struct Item {
    item: Option<mkt_data::Item>,
    i_type: Option<ItemType>,
    owned_item: Option<mkt_data::OwnedItem>,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    _inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Component for Item {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback_data = ctx.link().callback(Msg::DataStore);
        let callback_inv = ctx.link().callback(Msg::Inventory);
        Self {
            item: None,
            i_type: item_type_from_id(&ctx.props().id),
            owned_item: None,
            _data_store: DataStore::bridge(callback_data),
            _inventory: Inventory::bridge(callback_inv),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStore(state) => {
                if self.i_type.is_none() {
                    return false;
                }
                let i_type = self.i_type.unwrap();

                let state = state.borrow();

                let items = match i_type {
                    ItemType::Driver => &state.data.drivers,
                    ItemType::Kart => &state.data.karts,
                    ItemType::Glider => &state.data.gliders,
                };
                let item = items.get(&ctx.props().id);

                if item.map(|i| i.last_changed) != self.item.as_ref().map(|i| i.last_changed) {
                    self.item = item.cloned();
                    true
                } else {
                    false
                }
            }
            Msg::Inventory(state) => {
                if self.i_type.is_none() {
                    return false;
                }
                let i_type = self.i_type.unwrap();

                let state = state.borrow();

                let items = match i_type {
                    ItemType::Driver => &state.inv.drivers,
                    ItemType::Kart => &state.inv.karts,
                    ItemType::Glider => &state.inv.gliders,
                };
                let item = items.get(&ctx.props().id);

                if item.map(|i| i.last_changed) != self.item.as_ref().map(|i| i.last_changed) {
                    self.owned_item = item.cloned();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.i_type = item_type_from_id(&ctx.props().id);
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if let Some(item) = self.item.as_ref() {
            if let Some(owned_item) = self.owned_item.as_ref() {
                html! {
                    <div>
                        <p>{ &item.name }</p>
                        <ul>
                            <li>{ format!("Level: {}", owned_item.lvl) }</li>
                            <li>{ format!("Points: {}", owned_item.points) }</li>
                        </ul>
                    </div>
                }
            } else {
                html! {
                    <p>{ &item.name }</p>
                }
            }
        } else {
            html! {
                <p>{ "no_item" }</p>
            }
        }
    }
}
