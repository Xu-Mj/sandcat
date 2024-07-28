use std::mem::take;

use base64::prelude::*;
use fluent::{FluentBundle, FluentResource};
use gloo::utils::document;
use log::error;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Blob, FileReader, HtmlDivElement, MediaRecorder, MediaRecorderOptions, MediaStream};
use yew::{
    html, Callback, Classes, Component, Context, Html, NodeRef, ProgressEvent, Properties,
    TouchEvent,
};

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::{
    error::Error,
    model::{notification::Notification, voice::Voice},
    state::{I18nState, MobileState, Notify},
};
use utils::tr;

use crate::constant::{CANCEL, CANCEL_MOBILE, ERROR, PRESS, RECORD, SEND, SEND_MOBILE, STOP};

pub struct Recorder {
    mask_node: NodeRef,
    holder_node: NodeRef,
    voice_node: NodeRef,
    is_mobile: bool,
    media_recorder: Option<MediaRecorder>,
    on_data_available_closure: Option<Closure<dyn FnMut(JsValue)>>,
    on_error_closure: Option<Closure<dyn FnMut(JsValue)>>,
    on_stop_closure: Option<Closure<dyn FnMut(web_sys::Event)>>,
    reader_container: Option<Closure<dyn FnMut(ProgressEvent)>>,
    i18n: FluentBundle<FluentResource>,
    record_state: RecorderState,
    start_time: i64,
    // voice data
    data: Vec<u8>,
    // voice time in seconds
    time: u8,
    is_cancel: bool,
}

#[derive(Clone, Properties, PartialEq)]
pub struct RecorderProps {
    pub send_voice: Callback<Voice>,
}

pub enum RecorderMsg {
    TouchStart(TouchEvent),
    TouchMove(TouchEvent),
    TouchEnd(TouchEvent),
    Prepare,
    PrepareError(JsValue),
    Start(MediaStream),
    DataAvailable(Blob),
    ReadData(JsValue),
    RecordeComplete,
    // SendComplete,
    Stop,
    Cancel,
    Send,
}

#[derive(Debug, Clone, PartialEq)]
enum RecorderState {
    Static,
    Prepare,
    Recording,
    Error,
    Stop,
}

impl Component for Recorder {
    type Message = RecorderMsg;

    type Properties = RecorderProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let res = match I18nState::get().lang {
            LanguageType::ZhCN => zh_cn::RECORDER,
            LanguageType::EnUS => en_us::RECORDER,
        };
        let i18n = utils::create_bundle(res);
        Self {
            is_mobile: MobileState::is_mobile(),
            mask_node: NodeRef::default(),
            holder_node: NodeRef::default(),
            voice_node: NodeRef::default(),
            media_recorder: None,
            on_data_available_closure: None,
            on_error_closure: None,
            on_stop_closure: None,
            reader_container: None,
            i18n,
            record_state: RecorderState::Static,
            data: vec![],
            time: 0,
            start_time: 0,
            is_cancel: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RecorderMsg::Prepare => {
                // prepare audio stream
                self.record_state = RecorderState::Prepare;
                ctx.link().send_future(async {
                    match utils::get_audio_stream().await {
                        Ok(stream) => RecorderMsg::Start(stream),
                        Err(e) => RecorderMsg::PrepareError(e),
                    }
                });
                true
            }
            RecorderMsg::Start(stream) => {
                self.record_state = RecorderState::Recording;

                let mut options = MediaRecorderOptions::new();
                options.mime_type("audio/webm;codecs=opus");
                options.bits_per_second(32000);
                // todo handle error
                let recorder = match MediaRecorder::new_with_media_stream_and_media_recorder_options(
                    &stream, &options,
                ) {
                    Ok(recorder) => recorder,
                    Err(e) => {
                        error!("recorder error: {:?}", e);
                        ctx.link().send_message(RecorderMsg::PrepareError(e));
                        // Dialog::error("Recorder error");
                        return false;
                    }
                };

                let ctx_clone = ctx.link().clone();
                let on_data_available_closure = Closure::wrap(Box::new(move |data: JsValue| {
                    if let Ok(blob) = data.dyn_into::<web_sys::BlobEvent>() {
                        if let Some(blob) = blob.data() {
                            ctx_clone.send_message(RecorderMsg::DataAvailable(blob));
                        }
                    }
                })
                    as Box<dyn FnMut(JsValue)>);

                let ctx_clone = ctx.link().clone();
                let on_error_closure = Closure::wrap(Box::new(move |e: JsValue| {
                    error!("MediaRecorder error: {:?}", e);
                    ctx_clone.send_message(RecorderMsg::PrepareError(e));
                }) as Box<dyn FnMut(JsValue)>);

                recorder
                    .set_ondataavailable(Some(on_data_available_closure.as_ref().unchecked_ref()));
                recorder.set_onerror(Some(on_error_closure.as_ref().unchecked_ref()));

                self.on_error_closure = Some(on_error_closure);

                self.start_time = chrono::Utc::now().timestamp_millis();

                // start recording
                if let Err(e) = recorder.start() {
                    ctx.link().send_message(RecorderMsg::PrepareError(e));
                }
                self.on_data_available_closure = Some(on_data_available_closure);

                self.media_recorder = Some(recorder);

                // start animation
                self.start_voice_node_animation();
                true
            }
            RecorderMsg::DataAvailable(blob) => {
                let ctx = ctx.link().clone();
                let file_reader = match FileReader::new() {
                    Ok(reader) => reader,
                    Err(e) => {
                        error!("Error creating FileReader: {:?}", e);
                        // Dialog::error("Error creating FileReader");
                        ctx.send_message(RecorderMsg::PrepareError(e));
                        return false;
                    }
                };
                let onloadend_cb = Closure::wrap(Box::new(move |e: ProgressEvent| {
                    let file_reader: FileReader = e.target().unwrap().dyn_into().unwrap();
                    if let Ok(result) = file_reader.result() {
                        ctx.send_message(RecorderMsg::ReadData(result));
                    }
                })
                    as Box<dyn FnMut(ProgressEvent)>);

                file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
                self.reader_container = Some(onloadend_cb);
                file_reader.read_as_array_buffer(&blob).unwrap();
                false
            }
            RecorderMsg::ReadData(data) => {
                // calculate time
                self.time =
                    ((chrono::Utc::now().timestamp_millis() - self.start_time) / 1000) as u8;

                // read data
                if let Ok(buffer) = data.dyn_into::<js_sys::ArrayBuffer>() {
                    let uint_8_array = js_sys::Uint8Array::new(&buffer);
                    let mut audio_data_vec = uint_8_array.to_vec();
                    self.data.append(&mut audio_data_vec);
                }

                self.reader_container = None;
                ctx.link().send_message(RecorderMsg::RecordeComplete);
                false
            }
            RecorderMsg::Stop => {
                if let Some(ref recorder) = self.media_recorder {
                    recorder.stop().unwrap();
                }
                true
            }
            RecorderMsg::Cancel => {
                if let Some(ref recorder) = self.media_recorder {
                    recorder.stop().unwrap();
                    self.media_recorder = None;
                }
                self.record_state = RecorderState::Static;
                self.stop_voice_node_animation();
                self.data.clear();
                self.clean();
                true
            }
            RecorderMsg::Send => {
                self.record_state = RecorderState::Static;
                if self.time > 0 && !self.data.is_empty() {
                    self.send(ctx);
                }
                self.clean();
                true
            }
            RecorderMsg::PrepareError(e) => {
                web_sys::console::log_1(&e);
                error!("prepare error");
                self.record_state = RecorderState::Error;
                self.mask_node
                    .cast::<HtmlDivElement>()
                    .map(|div| div.style().set_property("display", "none"));
                self.clean();
                // Dialog::error(&tr!(self.i18n, "error"));
                // let msg = if let Some(err) = e.dyn_ref::<web_sys::DomException>() {
                //     err.name()
                // } else {
                //     tr!(self.i18n, ERROR)
                // };
                Notification::error(Error::js_err(e)).notify();
                true
            }
            RecorderMsg::TouchStart(event) => {
                event.stop_propagation();
                event.prevent_default();
                self.mask_node
                    .cast::<HtmlDivElement>()
                    .map(|div| div.style().remove_property("display"));
                // send message prepare to get audio stream
                ctx.link().send_message(RecorderMsg::Prepare);
                false
            }
            RecorderMsg::TouchEnd(event) => {
                event.stop_propagation();
                event.prevent_default();
                // send voice data
                if let Some(ref recorder) = self.media_recorder {
                    recorder.stop().unwrap();
                }
                false
            }
            RecorderMsg::TouchMove(event) => {
                event.stop_propagation();
                // check if the touch is out of the recorder
                if let Some(event) = event.changed_touches().get(0) {
                    let x = event.client_x() as f32;
                    let y = event.client_y() as f32;
                    if let Some(element) = document().element_from_point(x, y) {
                        if let Ok(div) = element.dyn_into::<HtmlDivElement>() {
                            if let Some(holder) = self.holder_node.cast::<HtmlDivElement>() {
                                self.is_cancel = !holder.eq(&div);
                            }
                        }
                    }
                }
                true
            }
            RecorderMsg::RecordeComplete => {
                if self.is_mobile {
                    if !self.is_cancel && self.time > 0 && !self.data.is_empty() {
                        self.send(ctx);
                    } else {
                        // todo warning voice too short
                    }

                    self.record_state = RecorderState::Static;
                    self.mask_node
                        .cast::<HtmlDivElement>()
                        .map(|div| div.style().set_property("display", "none"));
                    self.clean();
                }
                self.record_state = RecorderState::Stop;

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut error = html!();
        if self.record_state == RecorderState::Error {
            error = html! {
                <div class="error">
                    {tr!(self.i18n, ERROR)}
                </div>
            };
        }
        let content = if self.is_mobile {
            let touch_start = ctx.link().callback(RecorderMsg::TouchStart);
            let touch_end = ctx.link().callback(RecorderMsg::TouchEnd);
            let touch_move = ctx.link().callback(RecorderMsg::TouchMove);
            let voice = self.get_voice_html();

            // hint
            let hint = if self.is_cancel {
                tr!(self.i18n, CANCEL_MOBILE)
            } else {
                tr!(self.i18n, SEND_MOBILE)
            };
            html! {
                <div
                    ontouchstart={touch_start}
                    ontouchmove={touch_move}
                    ontouchend={touch_end}
                    class="recorder-mobile">

                    <div ref={self.mask_node.clone()}
                        class="recorder-mobile-mask"
                        style="display: none;">
                        {voice}
                        <div class="recorder-mobile-hint">{hint}</div>
                        <div ref={self.holder_node.clone()} class="recorder-holder">

                        </div>
                    </div>
                    {tr!(self.i18n, PRESS)}
                </div>
            }
        } else {
            let mut audio = html!();
            if self.record_state == RecorderState::Stop {
                let audio_base64 = BASE64_STANDARD.encode(&self.data);
                let data_url = format!("data:audio/mp3;base64,{}", audio_base64);
                audio = html!(<audio class="audio" src={data_url} controls={true}></audio>);
            };
            let mut voice = html!();

            let mut record_btn = true;
            let mut stop_btn = true;
            let mut send_btn = true;
            let mut cancel_btn = true;
            match self.record_state {
                RecorderState::Static => {
                    record_btn = false;
                    voice = self.get_voice_html();
                }
                RecorderState::Prepare => {
                    voice = self.get_voice_html();
                }
                RecorderState::Recording => {
                    stop_btn = false;
                    cancel_btn = false;
                    voice = self.get_voice_html();
                }
                RecorderState::Error => {}
                RecorderState::Stop => {
                    send_btn = false;
                    cancel_btn = false;
                }
            }

            let on_recorder_click = ctx.link().callback(|_| RecorderMsg::Prepare);
            html! {
                <div class="recorder">
                    <button class="btn" disabled={record_btn} onclick={on_recorder_click}>{tr!(self.i18n, RECORD)}</button>
                    {error}
                    {voice}
                    {audio}

                    <button class="btn" disabled={stop_btn} onclick={ctx.link().callback(|_| RecorderMsg::Stop)}>{tr!(self.i18n, STOP)}</button>
                    <button class="btn" disabled={send_btn} onclick={ctx.link().callback(|_| RecorderMsg::Send)}>{tr!(self.i18n, SEND)}</button>
                    <button class="btn" disabled={cancel_btn} onclick={ctx.link().callback(|_| RecorderMsg::Cancel)}>{tr!(self.i18n, CANCEL)}</button>
                </div>
            }
        };
        html! {
            {content}
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.clean();
    }
}

impl Recorder {
    fn send(&mut self, ctx: &Context<Self>) {
        let send_voice = ctx.props().send_voice.clone();
        let data = take(&mut self.data);
        let duration = self.time;
        let voice = Voice::new(nanoid::nanoid!(), data, duration);
        send_voice.emit(voice);
    }

    fn clean(&mut self) {
        self.media_recorder = None;
        self.time = 0;
        self.is_cancel = false;
        self.on_data_available_closure = None;
        self.on_stop_closure = None;
        self.on_error_closure = None;
    }

    fn stop_voice_node_animation(&self) {
        if let Some(voice_node) = self.voice_node.cast::<HtmlDivElement>() {
            let _ = voice_node
                .style()
                .set_property("animation-play-state", "paused");
        }
    }

    fn start_voice_node_animation(&self) {
        if let Some(voice_node) = self.voice_node.cast::<HtmlDivElement>() {
            let _ = voice_node
                .style()
                .set_property("animation-play-state", "running");
        }
    }

    fn get_voice_html(&self) -> Html {
        let mut voice = Vec::with_capacity(7);
        let heights = [1., 3., 2., 4., 1.5, 2.5, 3.];
        let times = [0.3, 0.6, 0.57, 0.52, 0.4, 0.3, 0.7];
        for i in 0..7 {
            let style = format!(
                "--voice-item-height: {}rem; --voice-item-animation-time:{}s",
                heights[i], times[i]
            );

            voice.push(html!(<div class="item" {style} />))
        }

        let mut class = Classes::from("voice");
        if self.is_mobile {
            class.push("voice voice-size-mobile");
        } else {
            class.push("voice voice-size");
        };

        if self.is_cancel {
            class.push("voice-cancel-background");
        } else {
            class.push("voice-normal-background");
        }
        html! {
            <div ref={self.voice_node.clone()} {class}>
                {voice}
            </div>

        }
    }
}
