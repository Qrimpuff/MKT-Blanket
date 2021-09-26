use mkt_data::{item_type_from_id, ItemId, ItemType};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::agents::data_inventory::{DataInvItem, DataInventory, DataInventoryAgent, Shared};
use crate::comps::modal_popup::view_item_modal;

use super::modal_popup::update_popup_layer;

#[derive(Clone)]
pub enum Msg {
    ToggleModal,
    DataInventory(Shared<DataInventory>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: ItemId,
}

pub struct Item {
    item: Option<Shared<DataInvItem>>,
    i_type: Option<ItemType>,
    visible: bool,
    _data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for Item {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataInventory);
        Self {
            item: None,
            i_type: item_type_from_id(&ctx.props().id),
            visible: false,
            _data_inventory: DataInventoryAgent::bridge(callback),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModal => {
                self.visible = !self.visible;
                update_popup_layer(self.visible);
                true
            }
            Msg::DataInventory(state) => {
                if self.i_type.is_none() {
                    return false;
                }
                let i_type = self.i_type.unwrap();

                let state = state.read().unwrap();

                let items = match i_type {
                    ItemType::Driver => &state.drivers,
                    ItemType::Kart => &state.karts,
                    ItemType::Glider => &state.gliders,
                };
                self.item = items.get(&ctx.props().id).cloned();
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.i_type = item_type_from_id(&ctx.props().id);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(item) = &self.item {
            let item = item.read().unwrap();
            html! {
                <>
                    <button class="button is-fullwidth" onclick={ctx.link().callback(|_| Msg::ToggleModal)}>
                        <span>{ &item.data.name }</span>
                        <span class="icon is-small ml-auto">
                            {
                                if let Some(inv) = &item.inv {
                                    if inv.lvl > 0 {
                                        html! {<i class="fas fa-check has-text-success"></i>}
                                    } else if inv.lvl == 7 {
                                        html! {<i class="fas fa-star has-text-success"></i>}
                                    } else {
                                        html! {}
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </span>
                    </button>
                    { view_item_modal(self.visible, &self.item, ctx, Msg::ToggleModal) }
                </>
            }
        } else {
            html! {
                <p>{ "no_item" }</p>
            }
        }
    }
}
