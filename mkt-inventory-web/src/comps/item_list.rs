use std::cmp::Reverse;

use mkt_data::ItemType;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    agents::data_inventory::{DataInvItem, DataInventory, DataInventoryAgent, Shared},
    comps::item::Item,
};

use super::item::ShowStat;

pub enum Msg {
    DataInventory(Shared<DataInventory>),
    ToggleDisplay,
    ShowStat(ShowStat),
    ToggleSort,
}

#[derive(Copy, Clone, PartialEq)]
enum SortOrder {
    Default,
    Up,
    Down,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub i_type: ItemType,
}

pub struct ItemList {
    items: Vec<Shared<DataInvItem>>,
    visible: bool,
    show_stat: ShowStat,
    sort: SortOrder,
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
            show_stat: ShowStat::Level,
            sort: SortOrder::Default,
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
                    self.sort_items();
                    true
                } else {
                    false
                }
            }
            Msg::ToggleDisplay => {
                self.visible = !self.visible;
                true
            }
            Msg::ShowStat(show_stat) => {
                if self.show_stat != show_stat {
                    self.show_stat = show_stat;

                    if self.sort != SortOrder::Default {
                        self.sort = SortOrder::Default;
                        self.sort_items();
                    }

                    true
                } else {
                    false
                }
            }
            Msg::ToggleSort => {
                self.sort = match self.sort {
                    SortOrder::Default => SortOrder::Down,
                    SortOrder::Up => SortOrder::Default,
                    SortOrder::Down => SortOrder::Up,
                };
                self.sort_items();
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
                <div class="buttons has-addons">
                    { self.view_sort_button(ctx, "Level", ShowStat::Level) }
                    { self.view_sort_button(ctx, "Favorites", ShowStat::FavoriteCourses) }
                    { self.view_sort_button(ctx, "Additional", ShowStat::AdditionalCourses) }
                </div>
                <div class="columns is-multiline">
                { for self.items.iter().map(|i| {
                    let i = i.read().unwrap();
                    html!{
                        <>
                        <div class="column is-one-quarter py-1">
                            <Item id={i.data.id.clone()} show_stat={self.show_stat}/>
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
                    <button class="button is-small" onclick={ctx.link().callback(|_| Msg::ToggleDisplay)}>{ if self.visible {'-'} else {'+'} }</button>
                </h2>
                { items }
            </>
        }
    }
}

impl ItemList {
    fn view_sort_button(&self, ctx: &Context<Self>, text: &str, show_stat: ShowStat) -> Html {
        let sort = match (self.show_stat == show_stat, self.sort) {
            (false, _) => html! {},
            (_, SortOrder::Default) => html! {},
            (_, SortOrder::Up) => {
                html! {<span class="icon is-small"><i class="fas fa-sort-up"></i></span>}
            }
            (_, SortOrder::Down) => {
                html! {<span class="icon is-small"><i class="fas fa-sort-down"></i></span>}
            }
        };
        html! {
            <button
                class={classes!("button", (self.show_stat == show_stat).then_some("is-info is-selected"))}
                onclick={
                    if self.show_stat == show_stat {
                        ctx.link().callback(|_| Msg::ToggleSort)
                    } else {
                        ctx.link().callback(move |_| Msg::ShowStat(show_stat))
                    }
                }>
                <span>{ text }</span>
                { sort }
            </button>
        }
    }

    fn sort_items(&mut self) {
        match self.sort {
            SortOrder::Default => {
                self.items.sort_by_key(|c| c.read().unwrap().data.sort);
            }
            SortOrder::Up => match self.show_stat {
                ShowStat::Level => {
                    self.items
                        .sort_by_key(|c| c.read().unwrap().inv.as_ref().map(|i| i.lvl));
                }
                ShowStat::FavoriteCourses => {
                    self.items.sort_by_key(|c| {
                        c.read()
                            .unwrap()
                            .stats
                            .as_ref()
                            .map(|s| (s.max_fav_course_count, s.fav_course_count))
                    });
                }
                ShowStat::AdditionalCourses => {
                    self.items.sort_by_key(|c| {
                        c.read()
                            .unwrap()
                            .stats
                            .as_ref()
                            .map(|s| (s.max_add_course_count, s.add_course_count))
                    });
                }
            },
            SortOrder::Down => match self.show_stat {
                ShowStat::Level => {
                    self.items
                        .sort_by_key(|c| Reverse(c.read().unwrap().inv.as_ref().map(|i| i.lvl)));
                }
                ShowStat::FavoriteCourses => {
                    self.items.sort_by_key(|c| {
                        Reverse(
                            c.read()
                                .unwrap()
                                .stats
                                .as_ref()
                                .map(|s| (s.max_fav_course_count, s.fav_course_count)),
                        )
                    });
                }
                ShowStat::AdditionalCourses => {
                    self.items.sort_by_key(|c| {
                        Reverse(
                            c.read()
                                .unwrap()
                                .stats
                                .as_ref()
                                .map(|s| (s.max_add_course_count, s.add_course_count)),
                        )
                    });
                }
            },
        }
    }
}
