use async_trait::async_trait;
use gloo_net::http::Request;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, FormData, Headers};
use web_sys::{File, Response};

use crate::api::file::FileApi;
use crate::api::{token, AUTHORIZE_HEADER};
use crate::error::{Error, Result};

use super::RespStatus;

pub struct FileHttp;

impl FileHttp {
    async fn upload_file_inner(&self, url: &str, file: &File) -> Result<String> {
        let form = FormData::new()?;
        form.append_with_blob("file", file)?;

        // create request body
        let mut opts = web_sys::RequestInit::new();
        opts.method("POST");
        opts.body(Some(&form));

        // set auth header
        let header = Headers::new()?;
        header.set(AUTHORIZE_HEADER, &token())?;

        opts.headers(&header.into());

        // create request
        let request = web_sys::Request::new_with_str_and_init(url, &opts)?;

        // send request
        let window = web_sys::window().ok_or(Error::NoWindow)?;
        let request_promise = window.fetch_with_request(&request);
        let res: Response = JsFuture::from(request_promise).await?.dyn_into()?;
        let text = JsFuture::from(res.text()?).await?;

        text.as_string().ok_or(Error::JsToStr)
    }
}

#[async_trait(?Send)]
impl FileApi for FileHttp {
    async fn upload_file(&self, file: &File) -> Result<String> {
        let url = "/api/file/upload";
        self.upload_file_inner(url, file).await
    }

    async fn upload_avatar(&self, file: &File) -> Result<String> {
        let url = "/api/file/avatar/upload";
        self.upload_file_inner(url, file).await
    }

    // todo add auth header
    async fn upload_voice(&self, data: &[u8]) -> Result<String> {
        // convert Vec<u8> to Blob
        // we can't use Uint8Array type to set Blob because it will change the data
        let u8_array = js_sys::Uint8Array::from(data);
        let array: js_sys::Array = js_sys::Array::new_with_length(1);
        array.set(0, u8_array.buffer().into());

        let mut options = BlobPropertyBag::new();
        options.type_("audio/webm;codecs=opus");
        let blob = Blob::new_with_u8_array_sequence_and_options(&array, &options)?;
        web_sys::console::log_1(&blob);
        let form = FormData::new()?;
        form.append_with_blob_and_filename("file", &blob, "audio.webm")?;

        let url = "/api/file/upload";
        let text = Request::post(url)
            .header(AUTHORIZE_HEADER, &token())
            .body(form)?
            .send()
            .await?
            .success()
            .await?
            .text()
            .await?;

        Ok(text)
    }

    async fn download_voice(&self, name: &str) -> Result<Vec<u8>> {
        let url = format!("/api/file/get/{}", name);
        let result = Request::get(&url)
            .header(AUTHORIZE_HEADER, &token())
            .send()
            .await?
            .success()
            .await?
            .binary()
            .await?;
        Ok(result)
    }
}
