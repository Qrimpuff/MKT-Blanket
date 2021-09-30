use mkt_data::ItemType;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    agents::data_inventory::{DataInvItem, DataInventory, DataInventoryAgent, Shared},
    comps::item::Item,
};

pub enum Msg {
    DataInventory(Shared<DataInventory>),
    Toggle,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub i_type: ItemType,
}

pub struct ItemList {
    items: Vec<Shared<DataInvItem>>,
    visible: bool,
    _data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for ItemList {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataInventory);
        Self {
            items: Vec::new(),
            visible: false,
            _data_inventory: DataInventoryAgent::bridge(callback),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataInventory(state) => {
                let state = state.read().unwrap();
                let items = match ctx.props().i_type {
                    ItemType::Driver => &state.drivers,
                    ItemType::Kart => &state.karts,
                    ItemType::Glider => &state.gliders,
                };
                if items.len() != self.items.len() {
                    self.items = items.values().cloned().collect();
                    self.items.sort_by_key(|c| c.read().unwrap().data.sort);
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
                <>
                <div class="columns is-multiline">
                { for self.items.iter().map(|i| {
                    let i = i.read().unwrap();
                    html!{
                        <>
                        <div class="column is-one-quarter py-1">
                            <Item id={i.data.id.clone()} />
                        </div>
                        </>
                    }
                }) }
                </div>
                </>
            }
        } else {
            html! {}
        };
        html! {
            <>
                <h2 class="subtitle">
                    { title }{" "}
                    <button class="button is-small" onclick={ctx.link().callback(|_| Msg::Toggle)}>{ if self.visible {'-'} else {'+'} }</button>
                </h2>
                { items }
            </>
        }
    }
}
