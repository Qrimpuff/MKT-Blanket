use gloo::events::EventListener;
use mkt_data::ItemType;
use yew::prelude::*;

use crate::comps::item_list::ItemList;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Page {
    Drivers,
    Karts,
    Gliders,
}

impl From<String> for Page {
    fn from(s: String) -> Self {
        match s.as_str() {
            "#items/drivers" => Page::Drivers,
            "#items/karts" => Page::Karts,
            "#items/gliders" => Page::Gliders,
            _ => Page::Drivers,
        }
    }
}

pub enum Msg {
    Nav(String),
}

pub struct ItemTabs {
    page: Page,
    _nav_listener: EventListener,
}

impl Component for ItemTabs {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // navigation
        let hash = yew::utils::window().location().hash().unwrap();
        let nav_cb = ctx.link().callback(Msg::Nav);
        let _nav_listener = EventListener::new(&yew::utils::window(), "popstate", move |_| {
            gloo::console::info!("from item tabs popstate");

            let hash = yew::utils::window().location().hash().unwrap();
            nav_cb.emit(hash)
        });
        Self {
            page: hash.into(),
            _nav_listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Nav(hash) => {
                let new_page = hash.into();
                if self.page != new_page {
                    self.page = new_page;
                    gloo::console::info!(format!("item tab: {:?}", self.page));
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let content = match self.page {
            Page::Drivers => html! {<ItemList i_type={ItemType::Driver}/>},
            Page::Karts => html! {<ItemList i_type={ItemType::Kart}/>},
            Page::Gliders => html! {<ItemList i_type={ItemType::Glider}/>},
        };
        html! {
            <>
            <h2 class="title is-4">{"Inventory"}</h2>
            <div class="block">
                <div class="tabs is-medium is-boxed">
                    <ul>
                        <li class={classes!((self.page == Page::Drivers).then_some("is-active"))}><a href="#items/drivers">{"Drivers"}</a></li>
                        <li class={classes!((self.page == Page::Karts).then_some("is-active"))}><a href="#items/karts">{"Karts"}</a></li>
                        <li class={classes!((self.page == Page::Gliders).then_some("is-active"))}><a href="#items/gliders">{"Gliders"}</a></li>
                    </ul>
                </div>
                {content}
            </div>
            </>
        }
    }
}
