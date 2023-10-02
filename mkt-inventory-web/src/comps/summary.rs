use crate::agents::data_inventory::Shared;
use crate::agents::data_inventory::{DataInventory, DataInventoryAgent};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

pub enum Msg {
    DataInventory(Shared<DataInventory>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct Summary {
    course_count: usize,
    driver_count: usize,
    kart_count: usize,
    glider_count: usize,
    course_covered_count: usize,
    driver_owned_count: usize,
    kart_owned_count: usize,
    glider_owned_count: usize,
    _data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for Summary {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataInventory);
        Self {
            course_count: 0,
            driver_count: 0,
            kart_count: 0,
            glider_count: 0,
            course_covered_count: 0,
            driver_owned_count: 0,
            kart_owned_count: 0,
            glider_owned_count: 0,
            _data_inventory: DataInventoryAgent::bridge(callback),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataInventory(state) => {
                let state = state.read().unwrap();

                self.course_count = state.courses.len();
                self.course_covered_count = 0;
                for course in state.courses.values() {
                    let course = course.read().unwrap();
                    if let Some(stats) = &course.stats {
                        if stats.driver_owned_count > 0
                            && stats.kart_owned_count > 0
                            && stats.glider_owned_count > 0
                        {
                            self.course_covered_count += 1;
                        }
                    }
                }

                self.driver_count = state.drivers.len();
                self.driver_owned_count = 0;
                for driver in state.drivers.values() {
                    let driver = driver.read().unwrap();
                    if let Some(inv) = &driver.inv {
                        if inv.lvl > 0 {
                            self.driver_owned_count += 1;
                        }
                    }
                }

                self.kart_count = state.karts.len();
                self.kart_owned_count = 0;
                for kart in state.karts.values() {
                    let kart = kart.read().unwrap();
                    if let Some(inv) = &kart.inv {
                        if inv.lvl > 0 {
                            self.kart_owned_count += 1;
                        }
                    }
                }

                self.glider_count = state.gliders.len();
                self.glider_owned_count = 0;
                for glider in state.gliders.values() {
                    let glider = glider.read().unwrap();
                    if let Some(inv) = &glider.inv {
                        if inv.lvl > 0 {
                            self.glider_owned_count += 1;
                        }
                    }
                }

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <h2 class="title is-4">{"Welcome"}</h2>
                <p class="block">{"MKT Blanket is a course coverage tool for Mario Kart Tour. You can maintain you inventory and check which course is missing coverage."}</p>
                <article class="message is-warning">
                    <div class="message-body">
                        {"No more updates are planned as of the 4th Anniversary of Mario Kart Tour. The coverage data will remain as is."}
                    </div>
                </article>
                <article class="message is-info">
                    <div class="message-body">
                        {"To import your inventory from in-game screenshots or to use your inventory in a spreadsheet, go to the "}<a href="#import">{"Import / Export"}</a>{" page."}
                    </div>
                </article>
                <div class="block">
                    <h3 class="title is-5">{ "Coverage Summary" }</h3>
                    <ul>
                        <li><b>{"Courses: "}</b>{ format!("{}/{} ({:.1}%)", self.course_covered_count, self.course_count, self.course_covered_count as f64 / self.course_count as f64 * 100.0) }</li>
                        <li><b>{"Drivers: "}</b>{ format!("{}/{} ({:.1}%)", self.driver_owned_count, self.driver_count, self.driver_owned_count as f64 / self.driver_count as f64 * 100.0) }</li>
                        <li><b>{"Karts: "}</b>{ format!("{}/{} ({:.1}%)", self.kart_owned_count, self.kart_count, self.kart_owned_count as f64 / self.kart_count as f64 * 100.0) }</li>
                        <li><b>{"Gliders: "}</b>{ format!("{}/{} ({:.1}%)", self.glider_owned_count, self.glider_count, self.glider_owned_count as f64 / self.glider_count as f64 * 100.0) }</li>
                    </ul>
                </div>
            </>
        }
    }
}
