mod comps;
mod agents;

use yew::prelude::*;

use crate::comps::{course_list::CourseList, item_list::ItemList, summary::Summary};

struct App {}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <Summary/>
                <CourseList/>
                <ItemList/>
                <ItemList/>
                <ItemList/>
            </>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
