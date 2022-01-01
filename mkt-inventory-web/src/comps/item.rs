use gloo::events::EventListener;
use itertools::Itertools;
use mkt_data::{
    course_parts_from_id, item_type_from_id, ItemId, ItemLvl, ItemPoints, ItemType, OwnedItem,
};
use yew::prelude::*;
use yew_agent::utils::store::{Bridgeable, StoreWrapper};
use yew_agent::{Bridge, Bridged};

use crate::agents::data_inventory::{
    DataInvItem, DataInventory, DataInventoryAgent, DataInventoryRequest, Shared,
};
use crate::agents::inventory::{Inventory, InventoryRequest};
use crate::comps::course::Course;

use super::modal_popup::*;

#[derive(Clone)]
pub enum Msg {
    ToggleModal,
    DeleteToggleModal,
    DeleteItem,
    EditToggle,
    SetLevel(ItemLvl),
    SetPoints(ItemPoints),
    IncrementPoints,
    DecrementPoints,
    AddItem,
    DataInventory(Shared<DataInventory>),
}

#[derive(Copy, Clone, PartialEq)]
pub enum ShowStat {
    Level,
    Points,
    LevelPoints,
    FavoriteCourses,
    AdditionalCourses,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: ItemId,
    #[prop_or(ShowStat::Level)]
    pub show_stat: ShowStat,
    #[prop_or(0)]
    pub lvl_req: u8,
}

pub struct Item {
    item: Option<Shared<DataInvItem>>,
    i_type: Option<ItemType>,
    visible: bool,
    delete_visible: bool,
    edit_visible: bool,
    popup_listener: Option<EventListener>,
    data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
    inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
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
            delete_visible: false,
            edit_visible: false,
            popup_listener: None,
            data_inventory: DataInventoryAgent::bridge(callback),
            inventory: Inventory::bridge(Callback::noop()),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModal => {
                self.visible = !self.visible;
                self.popup_listener = update_popup_layer(self.visible, ctx, Msg::ToggleModal);
                if !self.visible {
                    self.edit_visible = false;
                }
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
            Msg::DeleteToggleModal => {
                self.delete_visible = !self.delete_visible;
                self.popup_listener =
                    update_popup_layer(self.delete_visible, ctx, Msg::DeleteToggleModal);
                true
            }
            Msg::DeleteItem => {
                if let Some(item) = &self.item {
                    if let Some(item) = &item.read().unwrap().inv {
                        self.inventory
                            .send(InventoryRequest::RemoveItem(item.id.clone()));
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Msg::EditToggle => {
                self.edit_visible = !self.edit_visible;
                true
            }
            Msg::SetLevel(lvl) => {
                if let Some(item) = &self.item {
                    if let Some(inv) = &item.read().unwrap().inv {
                        if lvl != inv.lvl {
                            let new_inv = OwnedItem::new(inv.id.clone(), lvl, inv.points);
                            self.inventory
                                .send(InventoryRequest::AddItem(self.i_type.unwrap(), new_inv));
                            return true;
                        }
                    }
                }
                false
            }
            Msg::SetPoints(points) => {
                if let Some(item) = &self.item {
                    if let Some(inv) = &item.read().unwrap().inv {
                        if points != inv.points {
                            let new_inv = OwnedItem::new(inv.id.clone(), inv.lvl, points);
                            self.inventory
                                .send(InventoryRequest::AddItem(self.i_type.unwrap(), new_inv));
                            return true;
                        }
                    }
                }
                false
            }
            Msg::IncrementPoints => {
                if let Some(item) = &self.item {
                    let item = item.read().unwrap();
                    if let Some(inv) = &item.inv {
                        let mut inv = inv.clone();
                        inv.increment_points(&item.data);
                        ctx.link().send_message(Msg::SetPoints(inv.points));
                    }
                }
                false
            }
            Msg::DecrementPoints => {
                if let Some(item) = &self.item {
                    let item = item.read().unwrap();
                    if let Some(inv) = &item.inv {
                        let mut inv = inv.clone();
                        inv.decrement_points(&item.data);
                        ctx.link().send_message(Msg::SetPoints(inv.points));
                    }
                }
                false
            }
            Msg::AddItem => {
                if let Some(item) = &self.item {
                    let item = item.read().unwrap();
                    let new_inv = OwnedItem::new(
                        item.data.id.clone(),
                        *item.data.valid_levels().first().unwrap(),
                        *item.data.valid_points().first().unwrap(),
                    );
                    self.inventory
                        .send(InventoryRequest::AddItem(self.i_type.unwrap(), new_inv));
                    if !self.edit_visible {
                        ctx.link().send_message(Msg::EditToggle);
                    }
                    return true;
                }
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.i_type = item_type_from_id(&ctx.props().id);
        self.data_inventory.send(DataInventoryRequest::Refresh);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(item) = &self.item {
            let item = item.read().unwrap();
            let lvl = if let Some(lvl) = item.inv.as_ref().map(|i| i.lvl).filter(|lvl| *lvl > 0) {
                let class = if lvl >= ctx.props().lvl_req {
                    "stat-lvl"
                } else {
                    "stat-lvl-maybe"
                };
                html! {<span {class}>{ lvl }</span>}
            } else {
                html! {}
            };
            let points = if let Some(points) = item
                .inv
                .as_ref()
                .map(|i| i.points)
                .filter(|points| *points > 0)
            {
                html! {<span class="stat-points">{ points }</span>}
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
                    html! {<span class="stat">{ format!("{}/{}", fav, max_fav) }</span>}
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
                        <span>{if ctx.props().lvl_req > 1 { html! { <i class="ml-4">{format!(" Lvl. {}", ctx.props().lvl_req)}</i> } } else { html! {} }}</span>
                        <span class="ml-auto" style="padding-left: 0.5ch;">
                            { match ctx.props().show_stat {
                                ShowStat::Level => { lvl },
                                ShowStat::Points => { points },
                                ShowStat::LevelPoints => { html! {<>{lvl}<span class="stat-points-small">{points}</span></>} },
                                ShowStat::FavoriteCourses => { fav_count },
                                ShowStat::AdditionalCourses => { add_count },
                            } }
                        </span>
                    </button>
                    { self.view_item_modal(ctx) }
                </>
            }
        } else {
            html! {
                <p>{ "no_item" }</p>
            }
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        if self.delete_visible {
            self.popup_listener = update_popup_layer(false, ctx, Msg::DeleteToggleModal);
        }
        if self.visible {
            self.popup_listener = update_popup_layer(false, ctx, Msg::ToggleModal);
        }
    }
}

impl Item {
    fn view_item_modal(&self, ctx: &Context<Self>) -> Html {
        if self.visible {
            if let Some(item) = &self.item {
                let item = item.read().unwrap();
                let edit = html! {
                <>
                    <div class="field is-horizontal">
                        <div class="field-label is-normal">
                            <label>{"Level:"}</label>
                        </div>
                        <div class="field-body">
                            <div class="field is-narrow">
                                <div class="control">
                                    <div class="buttons">
                                        { for item.data.valid_levels().into_iter().map(|l| html! {
                                            <button class={classes!("button", (item.inv.as_ref().map(|i| i.lvl).unwrap_or_default() == l).then_some("is-info"))} onclick={ctx.link().callback(move |_| Msg::SetLevel(l))}>{l}</button>
                                        }) }
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="field is-horizontal">
                        <div class="field-label is-normal">
                            <label>{"Points:"}</label>
                        </div>
                        <div class="field-body">
                            <div class="field is-narrow">
                                <div class="control">
                                    <div class="buttons">
                                        <button class={classes!("button", "is-small")} onclick={ctx.link().callback(|_| Msg::DecrementPoints)}><span class="icon"><i class="fas fa-minus"/></span></button>
                                        { for item.data.points_cap_tiers().into_iter().map(|p| html! {
                                            <button class={classes!("button", (item.inv.as_ref().map(|i| i.points).unwrap_or_default() == p).then_some("is-info"))} onclick={ctx.link().callback(move |_| Msg::SetPoints(p))}>{p}</button>
                                        }) }
                                        <button class={classes!("button", "is-small")} onclick={ctx.link().callback(|_| Msg::IncrementPoints)}><span class="icon"><i class="fas fa-plus"/></span></button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </>
                };

                let inv = if let Some(inv) = &item.inv {
                    html! {
                        <div class="block">
                            <div class="block">
                                <span style="display: inline-block;min-width: 5rem;">{"Level:"}<span class="stat-lvl-big">{inv.lvl}</span></span>
                                <span>{"Points:"}<span class="stat-points-big">{inv.points}</span></span>
                            </div>
                            { if self.edit_visible { edit } else { html!{} }}
                            <div class="buttons">
                                <button class={classes!("button")} onclick={ctx.link().callback(|_| Msg::EditToggle)}>
                                    { if self.edit_visible {
                                        html!{<span class="icon"><i class="fas fa-minus-square"/></span>}
                                    } else {
                                        html!{<span class="icon"><i class="fas fa-edit"/></span>}
                                    }}
                                    <span>{ "Edit" }</span>
                                </button>
                                <button class={classes!("button", "is-danger")} onclick={ctx.link().callback(|_| Msg::DeleteToggleModal)}>
                                    <span class="icon"><i class="fas fa-trash-alt"/></span>
                                    <span>{ "Remove" }</span>
                                </button>
                            </div>
                        </div>
                    }
                } else {
                    html! {
                        <div class="block">
                            <p class="block">{"This item is not in your inventory."}</p>
                            <div class="buttons">
                                <button class={classes!("button", "is-success")} onclick={ctx.link().callback(|_| Msg::AddItem)}>
                                    <span class="icon"><i class="fas fa-plus"/></span>
                                    <span>{ "Add Item" }</span>
                                </button>
                            </div>
                        </div>
                    }
                };
                let mut favorite_courses = item.data.favorite_courses.iter().collect_vec();
                favorite_courses.sort_by_key(|i| course_parts_from_id(&i.id));
                let courses = html! {
                    <>
                        <p class="subtitle is-6">{"Coverage"}</p>
                        <div class="columns is-multiline" style="overflow-y: scroll; max-height: 60vh;">
                        { for favorite_courses.iter().map(|r| html!{
                            <div class="column is-full py-1"><Course id={r.id.clone()} lvl_req={r.lvl} i_type={item.data.i_type} /></div>
                        }) }
                        </div>
                    </>
                };

                let title = html! {
                    <>
                    <span class="icon rarity-dot">
                        {
                            match item.data.rarity {
                                mkt_data::Rarity::Normal => html! {<i class="fas fa-circle rarity-normal"></i>},
                                mkt_data::Rarity::Super => html! {<i class="fas fa-circle rarity-super"></i>},
                                mkt_data::Rarity::HighEnd => html! {<i class="fas fa-circle rarity-high-end"></i>},
                            }
                        }
                    </span>
                    <span>{ &item.data.name }</span>
                    </>
                };
                let content = html! {
                    <>
                    { inv }
                    { courses }
                    { view_confirm_modal(self.delete_visible,
                        Some(html!{ "Remove Item" }),
                        html!{ "Are you sure you want to remove this item from your inventory?" },
                        ctx, Msg::DeleteToggleModal, Msg::DeleteItem) }
                    </>
                };
                view_popup_modal(
                    self.visible,
                    Some(title),
                    content,
                    None,
                    ctx,
                    Msg::ToggleModal,
                )
            } else {
                html! {
                    <p>{ "no_item" }</p>
                }
            }
        } else {
            html! {}
        }
    }
}
