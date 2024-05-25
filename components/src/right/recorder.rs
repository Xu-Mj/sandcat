use std::mem::take;

use base64::prelude::*;
use fluent::{FluentBundle, FluentResource};
use gloo::timers::callback::Interval;
use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::state::I18nState;
use utils::tr;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Blob, FileReader, HtmlDivElement, MediaRecorder, MediaStream};
use yew::{
    html, Callback, Component, Context, Html, NodeRef, ProgressEvent, Properties, TouchEvent,
};
use yewdux::Dispatch;

pub struct Recorder {
    node_ref: NodeRef,
    voice_node: NodeRef,
    is_mobile: bool,
    show_mask: bool,
    media_recorder: Option<MediaRecorder>,
    on_data_available_closure: Option<Closure<dyn FnMut(JsValue)>>,
    on_error_closure: Option<Closure<dyn FnMut(JsValue)>>,
    reader_container: Vec<Closure<dyn FnMut(ProgressEvent)>>,
    i18n: FluentBundle<FluentResource>,
    record_state: RecorderState,
    // timer
    time_interval: Option<Interval>,
    // voice data
    data: Vec<u8>,
    // voice time in seconds
    time: u8,
}

#[derive(Clone, Properties, PartialEq)]
pub struct RecorderProps {
    pub send_voice: Callback<(Vec<u8>, u8)>,
}

pub enum RecorderMsg {
    TouchStart(TouchEvent),
    TouchEnd(TouchEvent),
    IncreaseTime,
    Prepare,
    PrepareError(JsValue),
    Start(MediaStream),
    DataAvailable(Blob),
    ReadData(JsValue),
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
        let res = match Dispatch::<I18nState>::global().get().lang {
            LanguageType::ZhCN => zh_cn::RECORDER,
            LanguageType::EnUS => en_us::RECORDER,
        };
        let i18n = utils::create_bundle(res);
        Self {
            is_mobile: false,
            node_ref: NodeRef::default(),
            voice_node: NodeRef::default(),
            show_mask: false,
            media_recorder: None,
            on_data_available_closure: None,
            on_error_closure: None,
            reader_container: vec![],
            i18n,
            time_interval: None,
            record_state: RecorderState::Static,
            data: vec![],
            time: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RecorderMsg::TouchStart(event) => {
                event.stop_propagation();
                event.prevent_default();
                log::debug!("touch start recorder");
                self.show_mask = true;
                self.node_ref
                    .cast::<HtmlDivElement>()
                    .map(|div| div.class_list().add_1("hover"));
                true
            }
            RecorderMsg::TouchEnd(event) => {
                event.stop_propagation();
                event.prevent_default();
                self.show_mask = false;
                true
            }
            RecorderMsg::Prepare => {
                // prepare audio stream
                // ctx.link().send_message(RecorderMsg::Preparing);
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
                log::debug!("start recorder");
                self.record_state = RecorderState::Recording;

                let ctx_clone = ctx.link().clone();
                // todo handle error
                let recorder = MediaRecorder::new_with_media_stream(&stream).unwrap();

                let on_data_available_closure = Closure::wrap(Box::new(move |data: JsValue| {
                    if let Ok(blob) = data.dyn_into::<web_sys::BlobEvent>() {
                        if let Some(blob) = blob.data() {
                            ctx_clone.send_message(RecorderMsg::DataAvailable(blob));
                        }
                    }
                })
                    as Box<dyn FnMut(JsValue)>);

                recorder
                    .set_ondataavailable(Some(on_data_available_closure.as_ref().unchecked_ref()));
                let ctx_clone = ctx.link().clone();
                let on_error_closure = Closure::wrap(Box::new(move |e: JsValue| {
                    log::error!("MediaRecorder error: {:?}", e);
                    ctx_clone.send_message(RecorderMsg::PrepareError(e));
                }) as Box<dyn FnMut(JsValue)>);

                recorder.set_onerror(Some(on_error_closure.as_ref().unchecked_ref()));
                self.on_error_closure = Some(on_error_closure);

                // start recording
                recorder.start_with_time_slice(500).unwrap();
                self.on_data_available_closure = Some(on_data_available_closure);

                self.media_recorder = Some(recorder);

                // start timer
                let ctx = ctx.link().clone();
                self.time_interval = Some(Interval::new(1000, move || {
                    ctx.send_message(RecorderMsg::IncreaseTime)
                }));

                // start animation
                self.start_voice_node_animation();
                true
            }
            RecorderMsg::DataAvailable(blob) => {
                let ctx = ctx.link().clone();
                let file_reader = FileReader::new().unwrap();
                let onloadend_cb = Closure::wrap(Box::new(move |e: ProgressEvent| {
                    let file_reader: FileReader = e.target().unwrap().dyn_into().unwrap();
                    if let Ok(result) = file_reader.result() {
                        ctx.send_message(RecorderMsg::ReadData(result));
                    }
                })
                    as Box<dyn FnMut(ProgressEvent)>);

                file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
                self.reader_container.push(onloadend_cb);
                file_reader.read_as_array_buffer(&blob).unwrap();
                false
            }
            RecorderMsg::ReadData(data) => {
                if let Ok(buffer) = data.dyn_into::<js_sys::ArrayBuffer>() {
                    let uint_8_array = js_sys::Uint8Array::new(&buffer);
                    let mut audio_data_vec = uint_8_array.to_vec();

                    // Store audio data in your model
                    self.data.append(&mut audio_data_vec);
                }
                false
            }
            RecorderMsg::Stop => {
                if let Some(ref recorder) = self.media_recorder {
                    recorder.stop().unwrap();
                    self.media_recorder = None;
                }
                self.reader_container.clear();
                self.record_state = RecorderState::Stop;
                self.time_interval = None;
                // self.on_data_available_closure = None;
                self.on_error_closure = None;
                true
            }
            RecorderMsg::Cancel => {
                if let Some(ref recorder) = self.media_recorder {
                    recorder.stop().unwrap();
                    self.media_recorder = None;
                }
                self.reader_container.clear();
                self.record_state = RecorderState::Static;
                self.time_interval = None;
                self.on_data_available_closure = None;
                self.on_error_closure = None;
                self.stop_voice_node_animation();
                self.data.clear();
                self.time = 0;
                true
            }
            RecorderMsg::Send => {
                self.time_interval = None;
                self.record_state = RecorderState::Static;
                if self.time > 0 {
                    ctx.props()
                        .send_voice
                        .emit((take(&mut self.data), self.time));
                }
                self.on_data_available_closure = None;
                self.on_error_closure = None;
                true
            }
            RecorderMsg::IncreaseTime => {
                self.time += 1;
                true
            }
            RecorderMsg::PrepareError(_) => {
                log::error!("prepare error");
                self.record_state = RecorderState::Error;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let touch_start = ctx.link().callback(RecorderMsg::TouchStart);
        let touch_end = ctx.link().callback(RecorderMsg::TouchEnd);

        let mut error = html!();
        if self.record_state == RecorderState::Error {
            error = html! {
                <div class="error">
                    {tr!(self.i18n, "error")}
                </div>
            };
        }
        let content = if self.is_mobile {
            html! {
                <div style="background-color: red;"
                    ref={self.node_ref.clone()}
                    ontouchstart={touch_start}
                    ontouchend={touch_end}
                    class="recorder">
                    {"按住讲话"}
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
                <div ref={self.node_ref.clone()} class="recorder">
                    <button class="btn" disabled={record_btn} onclick={on_recorder_click}>{tr!(self.i18n, "recorde")}</button>
                    {error}
                    {voice}
                    {audio}

                    <button class="btn" disabled={stop_btn} onclick={ctx.link().callback(|_| RecorderMsg::Stop)}>{tr!(self.i18n, "stop")}</button>
                    <button class="btn" disabled={send_btn} onclick={ctx.link().callback(|_| RecorderMsg::Send)}>{tr!(self.i18n, "send")}</button>
                    <button class="btn" disabled={cancel_btn} onclick={ctx.link().callback(|_| RecorderMsg::Cancel)}>{tr!(self.i18n, "cancel")}</button>
                </div>
            }
        };
        html! {
            {content}
        }
    }
}

impl Recorder {
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
        html! {
            <div ref={self.voice_node.clone()} class="voice">
                {voice}
            </div>

        }
    }
}
