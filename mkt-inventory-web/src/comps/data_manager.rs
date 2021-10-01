use gloo::file::Blob;
use wasm_bindgen::JsCast;
use web_sys::Url;
use yew::{prelude::*, utils::document};

use crate::comps::{
    delete_data::DeleteData, delete_hash::DeleteHash, delete_inv::DeleteInv,
    download_data::DownloadData, download_hash::DownloadHash, download_inv::DownloadInv,
    fetch_data::FetchData,
};

#[function_component(DataManager)]
pub fn view_data_manager() -> Html {
    html! {
        <div class="block">
            <h2 class="subtitle">{ "Data Manager" }</h2>
            <div class="buttons">
                <FetchData/>
            </div>
            <div class="buttons">
                <DownloadInv/>
                <DownloadHash/>
                <DownloadData/>
            </div>
            <div class="buttons">
                <DeleteInv/>
                <DeleteHash/>
                <DeleteData/>
            </div>
        </div>
    }
}

pub fn download_file(file_name: &str, content: &str) {
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
