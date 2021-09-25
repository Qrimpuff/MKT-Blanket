use mkt_data::{item_type_from_id, ItemId, ItemType};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    agents::data_inventory::{DataInvItem, DataInventory, DataInventoryAgent, Shared},
    comps::course::Course,
};

pub enum Msg {
    Toggle,
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
            Msg::Toggle => {
                self.visible = !self.visible;
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
        // prevent scrolling on modal, FIXME
        let _: Option<_> = try {
            web_sys::window()?
                .document()?
                .query_selector("html")
                .ok()??
                .set_class_name(self.visible.then_some("is-clipped").unwrap_or(""));
        };
        if let Some(item) = &self.item {
            let item = item.read().unwrap();
            let courses = if self.visible {
                html! {
                    <ul>
                    { for item.data.favorite_courses.iter().map(|r| html!{ <li><Course id={r.id.clone()} /></li> }) }
                    </ul>
                }
            } else {
                html! {}
            };
            html! {
                <>
                    <button class="button is-fullwidth" onclick={ctx.link().callback(|_| Msg::Toggle)}>
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
                    <div class={classes!("modal", self.visible.then_some("is-active"))}>
                        <div class="modal-background" onclick={ctx.link().callback(|_| Msg::Toggle)}></div>
                        <div class="modal-content">
                            <div class="box">
                                <div class="subtitle">{ &item.data.name }</div>
                                {
                                    if let Some(inv) = &item.inv {
                                        html! {
                                            <>
                                                <div>{ format!("Level: {}", inv.lvl) }</div>
                                                <div>{ format!("Points: {}", inv.points)}</div>
                                            </>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                                { courses }
                            </div>
                        </div>
                        <button class="modal-close is-large" aria-label="close" onclick={ctx.link().callback(|_| Msg::Toggle)}></button>
                    </div>
                </>
            }
        } else {
            html! {
                <p>{ "no_item" }</p>
            }
        }
    }
}
