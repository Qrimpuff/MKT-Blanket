use mkt_data::CourseId;
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::{agents::data::DataStore, comps::item::Item};

pub enum Msg {
    DataStore(ReadOnly<DataStore>),
    Toggle,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: CourseId,
}

pub struct Course {
    course: Option<mkt_data::Course>,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    visible: bool,
}

impl Component for Course {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataStore);
        Self {
            course: None,
            _data_store: DataStore::bridge(callback),
            visible: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStore(state) => {
                let state = state.borrow();

                let course = state.data.courses.get(&ctx.props().id);

                if course.map(|c| c.last_changed) != self.course.as_ref().map(|c| c.last_changed) {
                    self.course = course.cloned();
                    true
                } else {
                    false
                }
            }
            Msg::Toggle => {
                self.visible = !self.visible;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(course) = self.course.as_ref() {
            let items = if self.visible {
                html! {
                    <ul>
                    { for course.favorite_items.iter().map(|r| html!{ <li><Item id={r.id.clone()} /></li> }) }
                    </ul>
                }
            } else {
                html! {}
            };
            html! {
                <>
                    <button class="button is-fullwidth" onclick={ctx.link().callback(|_| Msg::Toggle)}>
                        { format!("{} {}", course.name, if self.visible {'-'} else {'+'}) }
                    </button>
                    <div class={classes!("modal", self.visible.then_some("is-active"))}>
                        <div class="modal-background" onclick={ctx.link().callback(|_| Msg::Toggle)}></div>
                        <div class="modal-content">
                            <div class="box">
                                { items }
                            </div>
                        </div>
                        <button class="modal-close is-large" aria-label="close" onclick={ctx.link().callback(|_| Msg::Toggle)}></button>
                    </div>
                </>
            }
        } else {
            html! {
                <p>{ "no_course" }</p>
            }
        }
    }
}
