use yew::prelude::*;

use super::{import_export_bgr::ImportExportBgr, import_screenshot::ImportScreenshot};

#[function_component(ImportExport)]
pub fn view_data_manager() -> Html {
    html! {
        <>
        <h2 class="title is-4">{"Import / Export"}</h2>
        <ImportScreenshot/>
        <ImportExportBgr/>
        </>
    }
}