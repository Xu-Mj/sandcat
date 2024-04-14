use fluent::{FluentBundle, FluentResource};
use gloo::utils::window;
use unic_langid::langid;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{MediaStream, MediaStreamConstraints, MediaStreamTrack};

/// 格式化时间
pub fn format_milliseconds(millis: i64) -> String {
    let duration = chrono::Duration::milliseconds(millis);

    let seconds = duration.num_seconds();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("时间: {:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("时间: {:02}:{:02}", minutes, seconds)
    }
}

/// 获取视频流
pub async fn get_video_stream() -> Result<MediaStream, JsValue> {
    let navigator = window().navigator();
    let devices = navigator.media_devices()?;
    // 获取视频
    let mut constraints = MediaStreamConstraints::new();
    constraints.video(&true.into());
    let media = devices.get_user_media_with_constraints(&constraints)?;
    log::debug!("media : {:?}", media);
    web_sys::console::log_1(&media);
    let value = JsFuture::from(media).await?;

    let video_stream = JsCast::dyn_into::<MediaStream>(value)?;
    // 获取音频
    let mut audio_constraints = MediaStreamConstraints::new();
    audio_constraints.audio(&true.into());
    let audio_value =
        match JsFuture::from(devices.get_user_media_with_constraints(&audio_constraints)?).await {
            Ok(audio_value) => audio_value,
            Err(_) => {
                // 在这里处理音频流获取失败的情况
                return Ok(video_stream);
            }
        };
    let audio_stream = JsCast::dyn_into::<MediaStream>(audio_value)?;

    // 将音频流加入到视频流
    for track in audio_stream.get_tracks() {
        if let Ok(track) = track.dyn_into::<MediaStreamTrack>() {
            video_stream.add_track(&track);
        }
    }
    Ok(video_stream)
}

/// 获取音频
pub async fn get_audio_stream() -> Result<MediaStream, JsValue> {
    let navigator = window().navigator();
    let devices = navigator.media_devices()?;
    // 获取音频
    let mut audio_constraints = MediaStreamConstraints::new();
    audio_constraints.audio(&true.into());
    let audio_value =
        JsFuture::from(devices.get_user_media_with_constraints(&audio_constraints)?).await?;
    let audio_stream = JsCast::dyn_into::<MediaStream>(audio_value)?;

    Ok(audio_stream)
}

#[macro_export]
macro_rules! tr {
    ($model:expr, $key:expr $(, $arg:expr)*) => {{
        let message = $model.get_message($key).expect("Message doesn't exist.");
        let pattern = message.value().expect("Message has no value.");

        let mut errors = Vec::new();
        $model
            .format_pattern(pattern, None, &mut errors)
            .to_string()
    }};
    ($model:expr, $key:expr, $($arg_name:expr => $arg_value:expr),* $(,)?) => {{
        let message = $model.get_message($key).expect("Message doesn't exist.");
        let pattern = message.value().expect("Message has no value.");

        let mut args = fluent_bundle::FluentArgs::new();
        $(
            args.set($arg_name, $arg_value);
        )*

        let mut errors = Vec::new();
        $model
            .format_pattern(pattern, Some(&args), &mut errors)
            .to_string()
    }};
}

pub fn create_bundle(content: impl Into<String>) -> FluentBundle<FluentResource> {
    let lang_id = langid!("en-US");
    let mut bundle = FluentBundle::new(vec![lang_id]);
    let resource =
        FluentResource::try_new(content.into()).expect("Failed to create FluentResource.");
    bundle
        .add_resource(resource)
        .expect("Failed to add FTL resources to the bundle.");

    bundle
}
