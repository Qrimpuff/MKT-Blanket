use mkt_data::ItemType;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    agents::data_inventory::{
        DataInvItem, DataInventory, DataInventoryAgent, DataInventoryRequest, Shared,
    },
    comps::item::Item,
};

use super::item::ShowStat;

pub enum Msg {
    DataInventory(Shared<DataInventory>),
    _ToggleDisplay,
    ShowStat(ShowStat),
    SortStat(SortStat),
}

#[derive(Copy, Clone, PartialEq)]
enum SortOrder {
    Up,
    Down,
}

#[derive(Copy, Clone, PartialEq)]
pub enum SortStat {
    Default,
    Name,
    Level,
    FavoriteCourses,
    AdditionalCourses,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub i_type: ItemType,
}

pub struct ItemList {
    items: Vec<Shared<DataInvItem>>,
    visible: bool,
    show_stat: ShowStat,
    sort_stat: SortStat,
    sort: SortOrder,
    data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for ItemList {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataInventory);
        Self {
            items: Vec::new(),
            visible: true,
            show_stat: ShowStat::Level,
            sort_stat: SortStat::Default,
            sort: SortOrder::Up,
            data_inventory: DataInventoryAgent::bridge(callback),
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
            Msg::_ToggleDisplay => {
                self.visible = !self.visible;
                true
            }
            Msg::ShowStat(show_stat) => {
                if self.show_stat != show_stat {
                    self.show_stat = show_stat;

                    true
                } else {
                    false
                }
            }
            Msg::SortStat(sort_stat) => {
                if self.sort_stat != sort_stat {
                    self.sort_stat = sort_stat;
                    self.sort = SortOrder::Up;
                } else {
                    self.sort = match self.sort {
                        SortOrder::Up => SortOrder::Down,
                        SortOrder::Down => SortOrder::Up,
                    };
                }
                match self.sort_stat {
                    SortStat::Level => self.show_stat = ShowStat::Level,
                    SortStat::FavoriteCourses => self.show_stat = ShowStat::FavoriteCourses,
                    SortStat::AdditionalCourses => self.show_stat = ShowStat::AdditionalCourses,
                    _ => {}
                }

                self.sort_items();
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        self.items.clear();
        self.data_inventory.send(DataInventoryRequest::Refresh);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let _title = match ctx.props().i_type {
            ItemType::Driver => "Drivers",
            ItemType::Kart => "Karts",
            ItemType::Glider => "Gliders",
        };
        let items = if self.visible {
            html! {
                <>
                <div class="buttons has-addons">
                    { self.view_sort_button(ctx, "Default", SortStat::Default) }
                    { self.view_sort_button(ctx, "Name", SortStat::Name) }
                    { self.view_sort_button(ctx, "Level", SortStat::Level) }
                    { self.view_sort_button(ctx, "Favorites", SortStat::FavoriteCourses) }
                    { self.view_sort_button(ctx, "Additional", SortStat::AdditionalCourses) }
                </div>
                <div class="buttons has-addons">
                    { self.view_show_button(ctx, "Level", ShowStat::Level) }
                    { self.view_show_button(ctx, "Favorites", ShowStat::FavoriteCourses) }
                    { self.view_show_button(ctx, "Additional", ShowStat::AdditionalCourses) }
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
                { items }
            </>
        }
    }
}

impl ItemList {
    fn view_show_button(&self, ctx: &Context<Self>, text: &str, show_stat: ShowStat) -> Html {
        html! {
            <button
                class={classes!("button", (self.show_stat == show_stat).then_some("is-info is-selected"))}
                onclick={ctx.link().callback(move |_| Msg::ShowStat(show_stat))}>
                <span>{ text }</span>
            </button>
        }
    }
    fn view_sort_button(&self, ctx: &Context<Self>, text: &str, sort_stat: SortStat) -> Html {
        let sort = match (self.sort_stat == sort_stat, self.sort) {
            (false, _) => html! {},
            (_, SortOrder::Up) => {
                html! {<span class="icon is-small"><i class="fas fa-sort-up"></i></span>}
            }
            (_, SortOrder::Down) => {
                html! {<span class="icon is-small"><i class="fas fa-sort-down"></i></span>}
            }
        };
        html! {
            <button
                class={classes!("button", (self.sort_stat == sort_stat).then_some("is-info is-selected"))}
                onclick={ctx.link().callback(move |_| Msg::SortStat(sort_stat))}>
                <span>{ text }</span>
                { sort }
            </button>
        }
    }

    fn sort_items(&mut self) {
        match self.sort_stat {
            SortStat::Default => self.items.sort_by_key(|c| c.read().unwrap().data.sort),
            SortStat::Name => self
                .items
                .sort_by_key(|c| c.read().unwrap().data.name.clone()),
            SortStat::Level => self
                .items
                .sort_by_key(|c| c.read().unwrap().inv.as_ref().map(|i| i.lvl)),
            SortStat::FavoriteCourses => self.items.sort_by_key(|c| {
                c.read()
                    .unwrap()
                    .stats
                    .as_ref()
                    .map(|s| (s.max_fav_course_count, s.fav_course_count))
            }),
            SortStat::AdditionalCourses => self.items.sort_by_key(|c| {
                c.read()
                    .unwrap()
                    .stats
                    .as_ref()
                    .map(|s| (s.max_add_course_count, s.add_course_count))
            }),
        };
        if matches!(self.sort, SortOrder::Down) {
            self.items.reverse();
        }
    }
}
