use yew::prelude::*;

#[function_component(SunIcon)]
pub fn sun_icon() -> Html {
    html! {
    <svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 24 24">
        <path fill="#000000" d="M11 5V1h2v4zm6.65 2.75l-1.375-1.375l2.8-2.875l1.4 1.425zM19 13v-2h4v2zm-8 10v-4h2v4zM6.35 7.7L3.5 4.925l1.425-1.4L7.75 6.35zm12.7 12.8l-2.775-2.875l1.35-1.35l2.85 2.75zM1 13v-2h4v2zm3.925 7.5l-1.4-1.425l2.8-2.8l.725.675l.725.7zM12 18q-2.5 0-4.25-1.75T6 12t1.75-4.25T12 6t4.25 1.75T18 12t-1.75 4.25T12 18m0-2q1.65 0 2.825-1.175T16 12t-1.175-2.825T12 8T9.175 9.175T8 12t1.175 2.825T12 16m0-4"/>
    </svg>
    }
}

#[function_component(MoonIcon)]
pub fn moon_icon() -> Html {
    html! {
    <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 256 256">
        <path fill="#000000" d="M238 96a6 6 0 0 1-6 6h-18v18a6 6 0 0 1-12 0v-18h-18a6 6 0 0 1 0-12h18V72a6 6 0 0 1 12 0v18h18a6 6 0 0 1 6 6m-94-42h10v10a6 6 0 0 0 12 0V54h10a6 6 0 0 0 0-12h-10V32a6 6 0 0 0-12 0v10h-10a6 6 0 0 0 0 12m71.25 100.28a6 6 0 0 1 1.07 6A94 94 0 1 1 95.76 39.68a6 6 0 0 1 7.94 6.79A90.11 90.11 0 0 0 192 154a91 91 0 0 0 17.53-1.7a6 6 0 0 1 5.72 1.98m-14.37 11.34q-4.42.38-8.88.38A102.12 102.12 0 0 1 90 64q0-4.45.38-8.88a82 82 0 1 0 110.5 110.5"/>
    </svg>
    }
}

#[function_component(CloseIcon)]
pub fn close_icon() -> Html {
    html! {
        <svg width="12" height="12" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M8 8L40 40" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M8 40L40 8" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}

#[function_component(BackIcon)]
pub fn back_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
            <path fill="none" stroke="#000000" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 6s-6 4.419-6 6s6 6 6 6" color="#000000"/>
        </svg>
    }
}

#[function_component(PlusRectIcon)]
pub fn plus_rect_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 256 256">
            <path fill="#666666" d="M208 36H48a12 12 0 0 0-12 12v160a12 12 0 0 0 12 12h160a12 12 0 0 0 12-12V48a12 12 0 0 0-12-12m4 172a4 4 0 0 1-4 4H48a4 4 0 0 1-4-4V48a4 4 0 0 1 4-4h160a4 4 0 0 1 4 4Zm-40-80a4 4 0 0 1-4 4h-36v36a4 4 0 0 1-8 0v-36H88a4 4 0 0 1 0-8h36V88a4 4 0 0 1 8 0v36h36a4 4 0 0 1 4 4"/>
        </svg>
    }
}

#[function_component(CatHeadIcon)]
pub fn cat_head_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 14 14">
            <path fill="none" stroke="#cccccc" stroke-linecap="round" stroke-linejoin="round" d="M6.5 10.25h1m-7 3l3.5-2m-3.5-3l4 1m2.98-6.97L12.87.77a.46.46 0 0 1 .46.11a.49.49 0 0 1 .16.45l-1.1 5.61c-.06-.15-.12-.31-.17-.47A5.75 5.75 0 0 0 7 2.25a5.75 5.75 0 0 0-5.22 4.22c-.05.17-.11.32-.17.48L.51 1.33A.49.49 0 0 1 .67.88a.46.46 0 0 1 .46-.11l5.39 1.51m1.98 10.8a6.7 6.7 0 0 1-3 0m8 .17l-3.5-2m3.5-3l-4 1"/>
        </svg>
    }
}

#[function_component(UpIcon)]
pub fn up_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
            <path fill="#000000" d="M8 19q-.213 0-.356-.144q-.144-.144-.144-.357q0-.212.144-.356Q7.788 18 8 18h8q.213 0 .356.144t.144.357q0 .212-.144.356Q16.213 19 16 19zm4-3.385q-.213 0-.357-.143q-.143-.144-.143-.357V6.883L8.715 9.662q-.14.14-.331.133q-.192-.007-.341-.153q-.137-.134-.137-.34t.14-.348l3.389-3.389q.13-.13.27-.183q.139-.053.298-.053q.159 0 .295.053q.137.053.267.183l3.408 3.408q.14.14.143.332q.003.191-.143.34q-.134.138-.34.138t-.348-.14L12.5 6.882v8.232q0 .213-.144.357q-.144.143-.357.143"/>
        </svg>
    }
}

#[function_component(CatFootIcon)]
pub fn cat_foot_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 256 256">
            <path fill="#cccccc" d="M240 108a28 28 0 1 1-28-28a28 28 0 0 1 28 28m-168 0a28 28 0 1 0-28 28a28 28 0 0 0 28-28m20-20a28 28 0 1 0-28-28a28 28 0 0 0 28 28m72 0a28 28 0 1 0-28-28a28 28 0 0 0 28 28m23.12 60.86a35.3 35.3 0 0 1-16.87-21.14a44 44 0 0 0-84.5 0A35.25 35.25 0 0 1 69 148.82A40 40 0 0 0 88 224a39.48 39.48 0 0 0 15.52-3.13a64.09 64.09 0 0 1 48.87 0a40 40 0 0 0 34.73-72Z"/>
        </svg>
    }
}

#[function_component(CycleIcon)]
pub fn cycle_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
            <path fill="#999999" d="M12 4a7.986 7.986 0 0 0-6.357 3.143L8 9.5H2v-6l2.219 2.219A9.982 9.982 0 0 1 12 2c5.523 0 10 4.477 10 10h-2a8 8 0 0 0-8-8m-8 8a8 8 0 0 0 14.357 4.857L16 14.5h6v6l-2.219-2.219A9.982 9.982 0 0 1 12 22C6.477 22 2 17.523 2 12z"/>
        </svg>
    }
}

#[function_component(AudioZoomInIcon)]
pub fn audio_zoom_in_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
            <path fill="#ffffff" d="m1.858 12.385l-.627-.627L4.483 8.5H1.615v-.885H6V12h-.885V9.133zM7.615 6V1.615H8.5v2.873l3.252-3.257l.633.632l-3.258 3.252H12V6zm6.44 15.77q-.503 0-.947-.187q-.445-.187-.812-.554L7.37 16.115l.481-.467q.227-.208.514-.268q.288-.06.582.012l2.785.687v-8.31q0-.212.144-.356t.356-.144q.213 0 .357.144q.143.144.143.356v9.652l-3.637-.983l3.916 3.897q.202.202.477.318q.275.116.569.116h3.675q1.056 0 1.778-.722t.722-1.778v-4q0-.212.144-.356t.356-.144q.213 0 .357.144q.143.144.143.356v4q0 1.458-1.021 2.48t-2.475 1.02zm.503-6.5v-4q0-.214.144-.358q.144-.143.356-.143q.213 0 .356.144q.144.144.144.356v4zm2.846 0v-3q0-.214.144-.358t.357-.143q.212 0 .356.144q.143.144.143.356v3zm-1.154 2.634"/>
        </svg>
    }
}

#[function_component(AudioZoomOutIcon)]
pub fn audio_zoom_out_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
            <path fill="#ffffff"
                d="M5.708 19H8.5q.213 0 .356.144T9 19.5q0 .212-.144.356Q8.713 20 8.5 20H4.808q-.344 0-.576-.232Q4 19.536 4 19.192V15.5q0-.213.144-.356q.144-.144.357-.144q.212 0 .356.144q.143.144.143.356v2.792l3.246-3.246q.14-.14.344-.15q.204-.01.364.15t.16.354t-.16.354zm12.584 0l-3.246-3.246q-.14-.14-.15-.344q-.01-.204.15-.364t.354-.16t.354.16L19 18.292V15.5q0-.213.144-.356T19.5 15q.212 0 .356.144q.143.144.143.356v3.692q0 .344-.232.576q-.232.232-.576.232H15.5q-.213 0-.356-.144Q15 19.712 15 19.5t.144-.356q.144-.143.356-.143zM5 5.708V8.5q0 .213-.144.356T4.499 9q-.212 0-.356-.144Q4 8.713 4 8.5V4.808q0-.344.232-.576Q4.464 4 4.808 4H8.5q.213 0 .356.144q.144.144.144.357q0 .212-.144.356Q8.713 5 8.5 5H5.708l3.246 3.246q.14.14.15.344q.01.204-.15.364t-.354.16t-.354-.16zm14 0l-3.246 3.246q-.14.14-.344.15q-.204.01-.364-.15t-.16-.354t.16-.354L18.292 5H15.5q-.213 0-.356-.144T15 4.499q0-.212.144-.356Q15.288 4 15.5 4h3.692q.344 0 .576.232q.232.232.232.576V8.5q0 .213-.144.356Q19.712 9 19.5 9t-.356-.144Q19 8.713 19 8.5z"/>
        </svg>
    }
}

#[function_component(MicrophoneIcon)]
pub fn microphone_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 36 36">
            <path fill="#000000" d="M18 24c3.9 0 7-3.1 7-7V9c0-3.9-3.1-7-7-7s-7 3.1-7 7v8c0 3.9 3.1 7 7 7M13 9c0-2.8 2.2-5 5-5s5 2.2 5 5v8c0 2.8-2.2 5-5 5s-5-2.2-5-5z" class="clr-i-outline clr-i-outline-path-1"/>
            <path fill="#000000" d="M30 17h-2c0 5.5-4.5 10-10 10S8 22.5 8 17H6c0 6.3 4.8 11.4 11 11.9V32h-3c-.6 0-1 .4-1 1s.4 1 1 1h8c.6 0 1-.4 1-1s-.4-1-1-1h-3v-3.1c6.2-.5 11-5.6 11-11.9" class="clr-i-outline clr-i-outline-path-2"/>
            <path fill="none" d="M0 0h36v36H0z"/>
        </svg>
    }
}

#[function_component(MicrophoneMuteIcon)]
pub fn microphone_mute_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 36 36">
            <path fill="#000000" d="M30 17h-2c0 1.8-.5 3.5-1.4 5l1.5 1.5c1.2-2 1.8-4.2 1.9-6.5" class="clr-i-outline clr-i-outline-path-1"/>
            <path fill="#000000" d="M18 4c2.8 0 5 2.2 5 5v8c0 .4-.1.8-.2 1.2l1.6 1.6c.4-.9.6-1.8.6-2.8V9c0-3.9-3.2-7-7.1-6.9c-2.9 0-5.6 1.9-6.5 4.7L13 8.3c.5-2.4 2.6-4.1 5-4.3" class="clr-i-outline clr-i-outline-path-2"/>
            <path fill="#000000" d="m25.2 26.6l6.9 6.9l1.4-1.4L4 2.6L2.6 4l8.4 8.4V17c0 3.9 3.1 7 7 7c1.3 0 2.5-.3 3.6-1l2.2 2.2C22.1 26.4 20.1 27 18 27c-5.4.2-9.8-4.1-10-9.4V17H6c.1 6.2 4.8 11.4 11 12v3h-3c-.6 0-1 .4-1 1s.4 1 1 1h8c.6 0 1-.4 1-1s-.4-1-1-1h-3v-3c2.2-.2 4.4-1 6.2-2.4m-11.4-6.9c-.5-.8-.8-1.7-.8-2.7v-2.6l7.1 7.1c-2.2 1-4.9.3-6.3-1.8" class="clr-i-outline clr-i-outline-path-3"/>
            <path fill="none" d="M0 0h36v36H0z"/>
        </svg>
    }
}

#[function_component(VolumeIcon)]
pub fn volume_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 24 24">
            <g fill="none" stroke="#000000">
                <path d="M3.158 13.93a3.752 3.752 0 0 1 0-3.86a1.5 1.5 0 0 1 .993-.7l1.693-.339a.45.45 0 0 0 .258-.153L8.17 6.395c1.182-1.42 1.774-2.129 2.301-1.938C11 4.648 11 5.572 11 7.42v9.162c0 1.847 0 2.77-.528 2.962c-.527.19-1.119-.519-2.301-1.938L6.1 15.122a.45.45 0 0 0-.257-.153L4.15 14.63a1.5 1.5 0 0 1-.993-.7z"/>
                <path stroke-linecap="round" d="M15.536 8.464a5 5 0 0 1 .027 7.044m4.094-9.165a8 8 0 0 1 .044 11.27"/>
            </g>
        </svg>
    }
}

#[function_component(VolumeMuteIcon)]
pub fn volume_mute_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 24 24">
            <g fill="none" stroke="#000000">
                <path d="M3.158 13.93a3.752 3.752 0 0 1 0-3.86a1.5 1.5 0 0 1 .993-.7l1.693-.339a.45.45 0 0 0 .258-.153L8.17 6.395c1.182-1.42 1.774-2.129 2.301-1.938C11 4.648 11 5.572 11 7.42v9.162c0 1.847 0 2.77-.528 2.962c-.527.19-1.119-.519-2.301-1.938L6.1 15.122a.45.45 0 0 0-.257-.153L4.15 14.63a1.5 1.5 0 0 1-.993-.7z"/>
                <path stroke-linecap="round" d="m15 15l6-6m0 6l-6-6"/>
            </g>
        </svg>
    }
}

#[function_component(VideoRecordIcon)]
pub fn video_record_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 24 24">
            <path fill="#000000" fill-rule="evenodd" d="M9.451 3.25h.098c1.602 0 2.872 0 3.876.119c1.03.122 1.88.377 2.588.96c.24.197.461.417.659.658c.582.709.837 1.557.96 2.588c.027.232.048.478.064.739c.786-.392 1.452-.714 2.007-.896c.652-.213 1.343-.299 1.98.095s.87 1.05.97 1.728c.097.655.097 1.516.097 2.551v.416c0 1.035 0 1.896-.097 2.55c-.1.679-.333 1.335-.97 1.729c-.637.394-1.328.308-1.98.095c-.555-.182-1.221-.504-2.007-.896c-.016.261-.037.507-.065.739c-.122 1.03-.377 1.88-.96 2.588c-.197.24-.417.461-.658.659c-.709.582-1.557.837-2.588.96c-1.005.118-2.274.118-3.876.118H9.45c-1.602 0-2.872 0-3.876-.119c-1.03-.122-1.88-.377-2.588-.96a4.751 4.751 0 0 1-.659-.658c-.582-.709-.837-1.557-.96-2.588c-.118-1.005-.118-2.274-.118-3.876V11.45c0-1.602 0-2.872.119-3.876c.122-1.03.377-1.88.96-2.588a4.75 4.75 0 0 1 .658-.659c.709-.582 1.557-.837 2.588-.96C6.58 3.25 7.85 3.25 9.451 3.25m6.799 9.25v-1c0-1.662-.001-2.843-.108-3.749c-.105-.889-.304-1.415-.63-1.813a3.256 3.256 0 0 0-.45-.45c-.398-.326-.924-.525-1.813-.63c-.906-.107-2.087-.108-3.749-.108s-2.843.001-3.749.108c-.889.105-1.415.304-1.813.63a3.25 3.25 0 0 0-.45.45c-.326.398-.525.924-.63 1.813c-.107.906-.108 2.087-.108 3.749v1c0 1.662.001 2.843.108 3.749c.105.889.304 1.415.63 1.813a3.3 3.3 0 0 0 .45.45c.398.326.924.525 1.813.63c.906.107 2.087.108 3.749.108s2.843-.001 3.749-.108c.889-.105 1.415-.304 1.813-.63a3.3 3.3 0 0 0 .45-.45c.326-.398.525-.924.63-1.813c.107-.906.108-2.087.108-3.749m1.5 1.537l.244.121c.995.498 1.666.831 2.176.998c.499.163.65.1.724.055c.074-.046.198-.153.275-.673c.079-.53.081-1.28.081-2.392v-.292c0-1.113-.002-1.862-.08-2.392c-.078-.52-.202-.627-.276-.673c-.074-.046-.225-.108-.724.055c-.51.167-1.18.5-2.176.998l-.244.122v2.67zM13.03 7.97a.75.75 0 1 0-1.06 1.06a.75.75 0 0 0 1.06-1.06m-2.12-1.061a2.25 2.25 0 1 1 3.182 3.182a2.25 2.25 0 0 1-3.182-3.182" clip-rule="evenodd"/>
        </svg>
    }
}

#[function_component(ImageIcon)]
pub fn image_icon() -> Html {
    html! {
        <svg width="20" height="20" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="xMidYMid meet">
            <path d="M10 44H38C39.1046 44 40 43.1046 40 42V14H30V4H10C8.89543 4 8 4.89543 8 6V42C8 43.1046 8.89543 44 10 44Z" fill="none" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M30 4L40 14" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <circle cx="18" cy="17" r="4" fill="none" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M15 28V37H33V21L23.4894 31.5L15 28Z" fill="none" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}

#[function_component(HangupIcon)]
pub fn hangup_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="3rem" height="3rem" viewBox="0 0 24 24">
            <path fill="#e11d48" d="M12 23c6.075 0 11-4.925 11-11S18.075 1 12 1S1 5.925 1 12s4.925 11 11 11M8.818 7.403L12 10.586l3.182-3.182l1.414 1.414L13.414 12l3.182 3.182l-1.415 1.414L12 13.414l-3.182 3.182l-1.415-1.414L10.586 12L7.403 8.818z"/>
        </svg>
    }
}

#[function_component(AnswerPhoneIcon)]
pub fn answer_phone_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1.07em" height="1em" viewBox="0 0 17 16">
            <g fill="#65a30d" fill-rule="evenodd">
                <path d="M10.878 3.259L8.366 5.398a.776.776 0 0 0 0 1.115l2.512 2.139c.339.331 1.122.452 1.122.048V6.939h3.991c.558 0 1.011-.424 1.011-.946s-.453-.946-1.011-.946H12v-1.74c0-.278-.748-.307-1.122-.048"/>
                <path d="M14.031 11.852c-.428-.539-1.123-1.32-1.718-1.394c-.362-.045-.778.255-1.188.538c-.08.04-.698.408-.773.43c-.396.113-1.241.146-1.752-.32c-.492-.45-1.27-1.283-1.898-2.046c-.6-.786-1.229-1.731-1.551-2.311c-.336-.601-.094-1.396.114-1.746c.038-.063.498-.536.601-.646l.015.018c.381-.32.78-.645.825-.997c.074-.586-.525-1.439-.953-1.979C5.325.858 4.662-.089 3.759.045c-.34.05-.633.169-.922.34L2.829.376l-.048.037l-.025.013l.003.004c-.166.128-.64.482-.694.53c-.586.521-1.468 1.748-.786 3.955c.506 1.64 1.585 3.566 3.055 5.514l-.008.007c.072.094.146.179.221.27c.07.093.139.185.211.277l.01-.007c1.56 1.879 3.196 3.381 4.689 4.267c2.01 1.192 3.439.655 4.099.228c.062-.041.534-.408.694-.529l.004.004c.006-.006.01-.014.018-.02a3.27 3.27 0 0 0 .043-.033l-.006-.008c.242-.234.436-.484.57-.799c.351-.829-.42-1.693-.848-2.234"/>
            </g>
        </svg>
    }
}

#[function_component(MsgPhoneIcon)]
pub fn msg_phone_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
            <path fill="#000000" d="m21.904 13.202l-.192.816a2.75 2.75 0 0 1-2.955 2.107l-1.754-.178A2.75 2.75 0 0 1 14.6 13.83l-.39-1.686a.25.25 0 0 0-.113-.157c-.321-.197-1.033-.32-2.097-.32c-.787 0-1.386.067-1.787.189c-.14.043-.238.085-.3.116l-.091.048l-.014.04l-.409 1.77a2.75 2.75 0 0 1-2.401 2.118l-1.746.177A2.75 2.75 0 0 1 2.3 14.03l-.195-.817a3.75 3.75 0 0 1 1.13-3.651C5.134 7.839 8.064 7 12 7c3.942 0 6.875.842 8.775 2.57a3.75 3.75 0 0 1 1.173 3.41zm-1.427-.514a2.25 2.25 0 0 0-.71-2.009C18.185 9.241 15.604 8.5 12 8.5c-3.598 0-6.177.739-7.76 2.172a2.25 2.25 0 0 0-.677 2.19l.195.818a1.25 1.25 0 0 0 1.342.953l1.746-.178a1.25 1.25 0 0 0 1.091-.962l.423-1.82l.043-.136c.375-.998 1.59-1.37 3.597-1.37c1.317 0 2.265.164 2.88.54c.401.245.687.642.792 1.1l.39 1.685c.12.522.559.909 1.091.963l1.755.178a1.25 1.25 0 0 0 1.343-.958l.192-.816z"/>
        </svg>
    }
}

#[function_component(HangupInNotifyIcon)]
pub fn hangup_in_notify_icon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
            <g fill="none">
                <path fill="#000000" d="M23 12.5L20.5 15l-3-2V8.842C15.976 8.337 14.146 8 12 8c-2.145 0-3.976.337-5.5.842V13l-3 2L1 12.5c.665-.997 2.479-2.657 5.5-3.658C8.024 8.337 9.855 8 12 8c2.146 0 3.976.337 5.5.842c3.021 1 4.835 2.66 5.5 3.658"/>
                <path stroke="#000000" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.5 8.842C15.976 8.337 14.146 8 12 8c-2.145 0-3.976.337-5.5.842m11 0c3.021 1 4.835 2.66 5.5 3.658L20.5 15l-3-2zm-11 0c-3.021 1-4.835 2.66-5.5 3.658L3.5 15l3-2z"/>
            </g>
        </svg>
    }
}

#[function_component(MaxIcon)]
pub fn max_icon() -> Html {
    html! {
       <svg width="12" height="12" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M42 8H6C4.89543 8 4 8.89543 4 10V38C4 39.1046 4.89543 40 6 40H42C43.1046 40 44 39.1046 44 38V10C44 8.89543 43.1046 8 42 8Z" fill="none" stroke="#000000" stroke-width="2"/>
        </svg>
    }
}

#[function_component(PlusIcon)]
pub fn plus_icon() -> Html {
    html! {
        <svg width="20" height="20" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M24.0605 10L24.0239 38" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M10 24L38 24" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}

#[function_component(PeoplePlusIcon)]
pub fn people_plus_icon() -> Html {
    html! {
        <svg width="20" height="20" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M19 20C22.866 20 26 16.866 26 13C26 9.13401 22.866 6 19 6C15.134 6 12 9.13401 12 13C12 16.866 15.134 20 19 20Z" fill="none" stroke="#000000" stroke-width="1" stroke-linejoin="round"/>
            <path fill-rule="evenodd" clip-rule="evenodd" d="M36 29V41V29Z" fill="none"/>
            <path fill-rule="evenodd" clip-rule="evenodd" d="M30 35H42H30Z" fill="none"/>
            <path d="M36 29V41M30 35H42" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M27 28H18.8C14.3196 28 12.0794 28 10.3681 28.8719C8.86278 29.6389 7.63893 30.8628 6.87195 32.3681C6 34.0794 6 36.3196 6 40.8V42H27" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}

#[function_component(ContactsIcon)]
pub fn contacts_icon() -> Html {
    html! {
        <svg class="icon" width="1rem" height="1rem" viewBox="0 0 16 16">
            <path fill="#000000" d="M15 14s1 0 1-1s-1-4-5-4s-5 3-5 4s1 1 1 1zm-7.978-1L7 12.996c.001-.264.167-1.03.76-1.72C8.312 10.629 9.282 10 11 10c1.717 0 2.687.63 3.24 1.276c.593.69.758 1.457.76 1.72l-.008.002l-.014.002zM11 7a2 2 0 1 0 0-4a2 2 0 0 0 0 4m3-2a3 3 0 1 1-6 0a3 3 0 0 1 6 0M6.936 9.28a6 6 0 0 0-1.23-.247A7 7 0 0 0 5 9c-4 0-5 3-5 4q0 1 1 1h4.216A2.24 2.24 0 0 1 5 13c0-1.01.377-2.042 1.09-2.904c.243-.294.526-.569.846-.816M4.92 10A5.5 5.5 0 0 0 4 13H1c0-.26.164-1.03.76-1.724c.545-.636 1.492-1.256 3.16-1.275ZM1.5 5.5a3 3 0 1 1 6 0a3 3 0 0 1-6 0m3-2a2 2 0 1 0 0 4a2 2 0 0 0 0-4"/>
        </svg>
    }
}

#[function_component(MessagesIcon)]
pub fn messages_icon() -> Html {
    html! {
        <svg class="icon" width="1rem" height="1rem" viewBox="0 0 512 512">
            <path fill="none" stroke="#000000" stroke-linecap="round" stroke-miterlimit="10" stroke-width="32" d="M431 320.6c-1-3.6 1.2-8.6 3.3-12.2a33.68 33.68 0 0 1 2.1-3.1A162 162 0 0 0 464 215c.3-92.2-77.5-167-173.7-167c-83.9 0-153.9 57.1-170.3 132.9a160.7 160.7 0 0 0-3.7 34.2c0 92.3 74.8 169.1 171 169.1c15.3 0 35.9-4.6 47.2-7.7s22.5-7.2 25.4-8.3a26.44 26.44 0 0 1 9.3-1.7a26 26 0 0 1 10.1 2l56.7 20.1a13.52 13.52 0 0 0 3.9 1a8 8 0 0 0 8-8a12.85 12.85 0 0 0-.5-2.7Z"/>
            <path fill="none" stroke="#000000" stroke-linecap="round" stroke-miterlimit="10" stroke-width="32" d="M66.46 232a146.23 146.23 0 0 0 6.39 152.67c2.31 3.49 3.61 6.19 3.21 8s-11.93 61.87-11.93 61.87a8 8 0 0 0 2.71 7.68A8.17 8.17 0 0 0 72 464a7.26 7.26 0 0 0 2.91-.6l56.21-22a15.7 15.7 0 0 1 12 .2c18.94 7.38 39.88 12 60.83 12A159.21 159.21 0 0 0 284 432.11"/>
        </svg>
    }
}

#[function_component(SettingIcon)]
pub fn setting_icon() -> Html {
    html! {
        <svg width="20" height="20" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="24" cy="12" r="3" fill="#000000"/>
        <circle cx="24" cy="24" r="3" fill="#000000"/>
        <circle cx="24" cy="35" r="3" fill="#000000"/>
    </svg>
     }
}

#[function_component(SearchIcon)]
pub fn search_icon() -> Html {
    html! {
    <svg class="icon" width="20" height="20" viewBox="0 0 48 48" fill="none">
        <path d="M21 38C30.3888 38 38 30.3888 38 21C38 11.6112 30.3888 4 21 4C11.6112 4 4 11.6112 4 21C4 30.3888 11.6112 38 21 38Z" fill="none" stroke="#000000" stroke-width="1" stroke-linejoin="round"/>
        <path d="M26.657 14.3431C25.2093 12.8954 23.2093 12 21.0001 12C18.791 12 16.791 12.8954 15.3433 14.3431" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M33.2216 33.2217L41.7069 41.707" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
    }
}

#[function_component(SmileIcon)]
pub fn smile_icon() -> Html {
    html! {
    <svg class="icon" width="20" height="20" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M24 44C35.0457 44 44 35.0457 44 24C44 12.9543 35.0457 4 24 4C12.9543 4 4 12.9543 4 24C4 35.0457 12.9543 44 24 44Z" fill="none" stroke="#000000" stroke-width="1" stroke-linejoin="round"/>
        <path d="M31 31C31 31 29 35 24 35C19 35 17 31 17 31" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M31 18V22" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M17 18V22" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
    }
}

#[function_component(FileIcon)]
pub fn file_icon() -> Html {
    html! {
    <svg class="icon" width="20" height="20" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M40 23V14L31 4H10C8.89543 4 8 4.89543 8 6V42C8 43.1046 8.89543 44 10 44H22" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M33 29V43" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M26 36H33H40" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M30 4V14H40" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
    }
}

#[function_component(FilePreviewIcon)]
pub fn file_preview_icon() -> Html {
    html! {
    <svg viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M40 23V14L31 4H10C8.89543 4 8 4.89543 8 6V42C8 43.1046 8.89543 44 10 44H22" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M28.2 30H37.8L41 34.1176L33 44L25 34.1176L28.2 30Z" fill="none" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M30 4V14H40" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
    }
}

#[function_component(PhoneIcon)]
pub fn phone_icon() -> Html {
    html! {
    <svg class="icon" width="20" height="20" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M16.9961 7.68583C17.7227 7.68583 18.3921 8.07985 18.7448 8.71509L21.1912 13.1219C21.5115 13.6989 21.5266 14.3968 21.2314 14.9871L18.8746 19.7008C18.8746 19.7008 19.5576 23.2122 22.416 26.0706C25.2744 28.929 28.7741 29.6002 28.7741 29.6002L33.487 27.2438C34.0777 26.9484 34.7761 26.9637 35.3533 27.2846L39.7726 29.7416C40.4072 30.0945 40.8008 30.7635 40.8008 31.4896L40.8008 36.5631C40.8008 39.1468 38.4009 41.0129 35.9528 40.1868C30.9249 38.4903 23.1202 35.2601 18.1734 30.3132C13.2265 25.3664 9.99631 17.5617 8.29977 12.5338C7.47375 10.0857 9.33984 7.68583 11.9235 7.68583L16.9961 7.68583Z"
            fill="none" stroke="#000000" stroke-width="1" stroke-linejoin="round"/>
    </svg>
    }
}

#[function_component(VideoIcon)]
pub fn video_icon() -> Html {
    html! {
    <svg class="icon" width="20" height="20" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
     <path d="M4 10C4 8.89543 4.89543 8 6 8H34C35.1046 8 36 8.89543 36 10V19L44 13V36L36 30V38C36 39.1046 35.1046 40 34 40H6C4.89543 40 4 39.1046 4 38V10Z" fill="none" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
     <circle cx="17" cy="21" r="5" fill="none" stroke="#000000" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
     }
}

#[function_component(SendMsgIcon)]
pub fn send_msg_icon() -> Html {
    html! {
       <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 256 256">
           <path fill="#90CBFB" d="m235.6 215.24l-8.09-28.32a76 76 0 0 0-60.75-110.61a76 76 0 1 0-138.27 62.61l-8.09 28.32A10 10 0 0 0 30 180a10.08 10.08 0 0 0 2.8-.4l28.32-8.09a76 76 0 0 0 28.13 8.18a76 76 0 0 0 105.71 39.82l28.32 8.09a10.08 10.08 0 0 0 2.8.4a10 10 0 0 0 9.56-12.76Zm-174.07-52a3.75 3.75 0 0 0-1.1.16l-29.87 8.53a2 2 0 0 1-2.47-2.47l8.53-29.87a4 4 0 0 0-.33-3a68 68 0 1 1 27.16 27.16a4 4 0 0 0-1.92-.53ZM227.4 219.4a2 2 0 0 1-2 .51l-29.87-8.53a4 4 0 0 0-3 .33A68 68 0 0 1 98 180a76 76 0 0 0 71.5-95.28a68 68 0 0 1 50.21 99.88a4 4 0 0 0-.33 3l8.53 29.87a2 2 0 0 1-.51 1.93"/>
       </svg>
    }
}
