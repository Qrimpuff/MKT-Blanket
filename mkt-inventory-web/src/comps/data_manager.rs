use yew::prelude::*;

use crate::comps::{delete_data::DeleteData, delete_inv::DeleteInv, fetch_data::FetchData};

#[function_component(DataManager)]
pub fn view_data_manager() -> Html {
    html! {
        <div class="block">
            <h2 class="subtitle">{ "Data Manager" }</h2>
            <div class="buttons">
                <FetchData/>
                <DeleteInv/>
                <DeleteData/>
            </div>
        </div>
    }
}
