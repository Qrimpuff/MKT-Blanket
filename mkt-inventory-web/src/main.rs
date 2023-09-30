#![feature(try_blocks)]
// rust-analyser doesn't like gloo::console
#![allow(unused_unsafe)]
#![allow(clippy::let_unit_value)]

mod agents;
mod comps;

use agents::{
    data::{DataRequest, DataStore},
    inventory::{Inventory, InventoryRequest},
    update::{UpdateAgent, UpdateRequest},
};
use gloo::events::EventListener;
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, StoreWrapper},
    Bridge, Bridged,
};

use crate::comps::{
    about::About, course_list::CourseList, data_manager::*, import_export::ImportExport,
    item_tabs::ItemTabs, summary::Summary,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Page {
    Main,
    Courses,
    Items,
    ImportExport,
    DataManager,
    About,
}

impl From<String> for Page {
    fn from(s: String) -> Self {
        match s.as_str() {
            "#" => Page::Main,
            "#courses" => Page::Courses,
            "#items" => Page::Items,
            "#items/drivers" => Page::Items,
            "#items/karts" => Page::Items,
            "#items/gliders" => Page::Items,
            "#import" => Page::ImportExport,
            "#data" => Page::DataManager,
            "#about" => Page::About,
            _ => Page::Main,
        }
    }
}

pub enum Msg {
    Nav(String),
    ToggleBurger,
    CloseBurger,
}

struct App {
    page: Page,
    burger: bool,
    _nav_listener: EventListener,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    _inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
    _update: Box<dyn Bridge<UpdateAgent>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // navigation
        let hash = gloo_utils::window().location().hash().unwrap();
        let nav_cb = ctx.link().callback(Msg::Nav);
        let _nav_listener = EventListener::new(&gloo_utils::window(), "popstate", move |_| {
            gloo::console::info!("from navigation popstate");

            let hash = gloo_utils::window().location().hash().unwrap();
            nav_cb.emit(hash)
        });
        // check for data updates
        let mut _update = UpdateAgent::bridge(Callback::noop());
        _update.send(UpdateRequest::CheckUpdateData);
        // initial load
        let mut _data_store = DataStore::bridge(Callback::noop());
        _data_store.send(DataRequest::Load);
        let mut _inventory = Inventory::bridge(Callback::noop());
        _inventory.send(InventoryRequest::Load);
        Self {
            page: hash.into(),
            burger: false,
            _nav_listener,
            _data_store,
            _inventory,
            _update,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Nav(hash) => {
                ctx.link().send_message(Msg::CloseBurger);
                let new_page = hash.into();
                if self.page != new_page {
                    self.page = new_page;
                    gloo::console::info!(format!("page: {:?}", self.page));
                    true
                } else {
                    false
                }
            }
            Msg::ToggleBurger => {
                self.burger = !self.burger;
                true
            }
            Msg::CloseBurger => {
                if self.burger {
                    self.burger = false;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = match self.page {
            Page::Main => html! {<Summary/>},
            Page::Courses => html! {<CourseList/>},
            Page::Items => html! {<ItemTabs/>},
            Page::ImportExport => html! {<ImportExport/>},
            Page::DataManager => html! {<DataManager/>},
            Page::About => html! {<About/>},
        };
        html! {
            <section class="section pt-4">
                <nav class="navbar is-fixed-top mkt-navbar" role="navigation" aria-label="main navigation">
                    <div class="navbar-brand">
                        <a class="navbar-item" href="/MKT-Blanket/">
                            <h1 class="title is-4">{ "MKT Blanket" }</h1>
                        </a>

                        <div role="button" class={classes!("navbar-burger", self.burger.then_some("is-active"))} data-target="navMenu" aria-label="menu" aria-expanded="false"
                            onclick={ctx.link().callback(|_| Msg::ToggleBurger)}>
                            <span aria-hidden="true"></span>
                            <span aria-hidden="true"></span>
                            <span aria-hidden="true"></span>
                        </div>
                    </div>
                    <div class={classes!("navbar-menu", self.burger.then_some("is-active"))} id="navMenu">
                        <a class="navbar-item" href="#home">{"Home"}</a>
                        <a class="navbar-item" href="#courses">{"Coverage"}</a>
                        <a class="navbar-item" href="#items/drivers">{"Inventory"}</a>
                        <a class="navbar-item" href="#import">{"Import / Export"}</a>
                        <a class="navbar-item" href="#data">{"Data Management"}</a>
                        <a class="navbar-item" href="#about">{"About"}</a>
                    </div>
                </nav>
                <div class="container is-clipped px-2 pb-4" onclick={ctx.link().callback(|_| Msg::CloseBurger)}>
                    {content}
                </div>
            </section>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
