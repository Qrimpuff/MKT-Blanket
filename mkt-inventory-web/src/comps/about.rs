use yew::prelude::*;

#[function_component(About)]
pub fn view_about() -> Html {
    html! {
        <div class="content">
            <h2 class="title is-4">{"About"}</h2>
            <p><b>{"GitHub: "}</b><a href="https://github.com/Qrimpuff/MKT-Blanket" target="_blank">{"https://github.com/Qrimpuff/MKT-Blanket"}</a></p>
            <p>{"The information used in this application comes from the "}<a href="https://www.mariowiki.com/" target="_blank">{"Super Mario Wiki"}</a>{" and is fetched daily. If you see incorrect information, please contribute to the wiki at these pages:"}</p>
            <ul>
                <li><b>{"Courses coverage: "}</b><a href="https://www.mariowiki.com/List_of_favored_and_favorite_courses_in_Mario_Kart_Tour" target="_blank">{"https://www.mariowiki.com/List_of_favored_and_favorite_courses_in_Mario_Kart_Tour"}</a></li>
                <li><b>{"Drivers: "}</b><a href="https://www.mariowiki.com/List_of_drivers_in_Mario_Kart_Tour" target="_blank">{"https://www.mariowiki.com/List_of_drivers_in_Mario_Kart_Tour"}</a></li>
                <li><b>{"Karts: "}</b><a href="https://www.mariowiki.com/List_of_karts_in_Mario_Kart_Tour" target="_blank">{"https://www.mariowiki.com/List_of_karts_in_Mario_Kart_Tour"}</a></li>
                <li><b>{"Gliders: "}</b><a href="https://www.mariowiki.com/List_of_gliders_in_Mario_Kart_Tour" target="_blank">{"https://www.mariowiki.com/List_of_gliders_in_Mario_Kart_Tour"}</a></li>
            </ul>
        </div>
    }
}
