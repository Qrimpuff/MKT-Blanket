#![feature(bool_to_option)]
#![feature(try_blocks)]

// rust-analyser doesn't like gloo::console
#![allow(unused_unsafe)]

mod agents;
mod comps;

use mkt_data::ItemType;
use yew::prelude::*;

use crate::comps::{
    course_list::CourseList, fetch_data::FetchData, import::Import, item_list::ItemList,
    summary::Summary,
};

struct App {}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="container is-clipped px-2">
                <h1 class="title">{ "MKT Inventory" }</h1>
                <FetchData/>
                <Summary/>
                <CourseList/>
                <ItemList i_type={ItemType::Driver}/>
                <ItemList i_type={ItemType::Kart}/>
                <ItemList i_type={ItemType::Glider}/>
                <Import/>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
