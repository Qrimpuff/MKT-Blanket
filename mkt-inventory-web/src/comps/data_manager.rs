use gloo::file::{Blob, BlobContents};
use gloo_utils::document;
use wasm_bindgen::JsCast;
use web_sys::Url;
use yew::prelude::*;

use crate::comps::{
    bootstrap_hash::BootstrapHash, delete_data::DeleteData, delete_hash::DeleteHash,
    delete_inv::DeleteInv, download_data::DownloadData, download_hash::DownloadHash,
    download_inv::DownloadInv, fetch_data::FetchData,
};

#[function_component(DataManager)]
pub fn view_data_manager() -> Html {
    html! {
        <>
        <h2 class="title is-4">{"Data Management"}</h2>
        <div class="block">
            <h3 class="subtitle is-4">{"Game Data"}</h3>
            <div class="buttons">
                <FetchData/>
                <DownloadData/>
                <DeleteData/>
            </div>
            <h3 class="subtitle is-4">{"Inventory"}</h3>
            <div class="buttons">
                <DownloadInv/>
                <DeleteInv/>
            </div>
            <h3 class="subtitle is-4">{"Item Hashes"}</h3>
            <div class="buttons">
                <DownloadHash/>
                <DeleteHash/>
            </div>
            <BootstrapHash/>
        </div>
        </>
    }
}

pub fn download_file(file_name: &str, content: impl BlobContents) {
    let a = document()
        .create_element("a")
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    document().body().unwrap().append_child(&a).unwrap();
    a.set_class_name("is-hidden");
    let blob = Blob::new_with_options(content, Some("octet/stream"));
    let url = Url::create_object_url_with_blob(&blob.into()).unwrap();
    a.set_attribute("href", &url).unwrap();
    a.set_attribute("download", file_name).unwrap();
    a.click();
    Url::revoke_object_url(&url).unwrap();
    document().body().unwrap().remove_child(&a).unwrap();
}
