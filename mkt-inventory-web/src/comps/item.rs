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

#[derive(Clone, PartialEq)]
pub enum ShowStat {
    Level,
    FavoriteCourses,
    AdditionalCourses,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: ItemId,
    #[prop_or(ShowStat::Level)]
    pub show_stat: ShowStat,
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
            let lvl = if let Some(lvl) = item.inv.as_ref().map(|i| i.lvl).filter(|lvl| *lvl > 0) {
                html! {<span class="stat-lvl">{ lvl }</span>}
            } else {
                html! {}
            };
            let fav_count = if let Some((fav, max_fav)) = item
                .stats
                .as_ref()
                .map(|s| (s.fav_course_count, s.max_fav_course_count))
            {
                if fav == max_fav {
                    html! {<span class="stat">{ fav }</span>}
                } else {
                    html! {<span class="stat">{ format!("{}-{}", fav, max_fav) }</span>}
                }
            } else {
                html! {}
            };
            let add_count = if let Some((add, max_add)) = item
                .stats
                .as_ref()
                .map(|s| (s.add_course_count, s.max_add_course_count))
            {
                if add == max_add {
                    html! {<span class="stat">{ add }</span>}
                } else {
                    html! {<span class="stat">{ format!("{}-{}", add, max_add) }</span>}
                }
            } else {
                html! {}
            };
            html! {
                <>
                    <button class="button is-fullwidth" onclick={ctx.link().callback(|_| Msg::ToggleModal)}>
                        <span class="icon rarity-dot">
                            {
                                match item.data.rarity {
                                    mkt_data::Rarity::Normal => html! {<i class="fas fa-circle rarity-normal"></i>},
                                    mkt_data::Rarity::Super => html! {<i class="fas fa-circle rarity-super"></i>},
                                    mkt_data::Rarity::HighEnd => html! {<i class="fas fa-circle rarity-high-end"></i>},
                                }
                            }
                        </span>
                        <span class="is-clipped-ellipsis">{ &item.data.name }</span>
                        <span class="ml-auto">
                            { match ctx.props().show_stat {
                                ShowStat::Level => { lvl },
                                ShowStat::FavoriteCourses => { fav_count },
                                ShowStat::AdditionalCourses => { add_count },
                            } }
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
