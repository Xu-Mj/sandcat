use std::collections::HashMap;

use gloo::utils::document;
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, JsValue};
use web_sys::{File, FileReader, HtmlCanvasElement, HtmlImageElement, HtmlInputElement, ImageData};
use yew::prelude::*;

pub struct Avatar {
    canvas_ref: NodeRef,
    reader: Option<FileReader>,
    avatar_onload: Option<Closure<dyn FnMut()>>,
    // avatar_onload: Option<Closure<dyn FnMut()>>,
    avatar_file: Option<File>,
    img: Option<HtmlImageElement>,
    scale: f64,
    x: f64,
    y: f64,
    dragging: bool,
    start_x: f64,
    start_y: f64,
    selection_size: f64,
    touches: HashMap<i32, (f64, f64)>,
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub submit: SubmitOption,
    pub close: Callback<()>,
    #[prop_or(256.0)]
    pub selection_size: f64,
    #[prop_or_default]
    pub avatar_url: Option<String>,
    #[prop_or("Choose".to_string())]
    pub choose_text: String,
    #[prop_or("Submit".to_string())]
    pub submit_text: String,
    #[prop_or("Cancel".to_string())]
    pub cancel_text: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SubmitOption {
    ImageData(Callback<ImageData>),
    DataUrl(Callback<String>),
    Data(Callback<Vec<u8>>),
}

pub enum Msg {
    Files(Event),
    Loaded(String),
    Wheel(WheelEvent),
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseMove(MouseEvent),
    TouchStart(TouchEvent),
    TouchMove(TouchEvent),
    TouchEnd(TouchEvent),
    SubmitSelection,
    ImageLoaded,
}

impl Component for Avatar {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let selection_size = ctx.props().selection_size;
        Self {
            canvas_ref: NodeRef::default(),
            reader: None,
            img: None,
            avatar_file: None,
            avatar_onload: None,
            scale: 1.0,
            x: 0.0,
            y: 0.0,
            dragging: false,
            start_x: 0.0,
            start_y: 0.0,
            selection_size,
            touches: HashMap::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Files(event) => {
                let file_input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
                let file_list = file_input.files();
                if let Some(file_list) = file_list {
                    if let Some(file) = file_list.get(0) {
                        let file_reader = FileReader::new().unwrap();
                        let reader = file_reader.clone();
                        let ctx = ctx.link().clone();
                        let on_load = Closure::wrap(Box::new(move || {
                            let result = reader.result().unwrap();
                            ctx.send_message(Msg::Loaded(result.as_string().unwrap()));
                        }) as Box<dyn FnMut()>);
                        file_reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
                        file_reader.read_as_data_url(&file).unwrap();
                        self.reader = Some(file_reader);
                        self.avatar_file = Some(file);
                        self.avatar_onload = Some(on_load);
                    }
                }
                false
            }
            Msg::Loaded(url) => {
                self.load_img(ctx, &url);
                false
            }
            Msg::Wheel(event) => {
                event.prevent_default();
                event.stop_propagation();
                let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
                if let Some(img) = &self.img {
                    let delta = event.delta_y();
                    let mouse_x = event.client_x() as f64;
                    let mouse_y = event.client_y() as f64;

                    // 计算新的缩放比例
                    let new_scale =
                        (self.scale + delta * -1_f64 / (img.width() * 5) as f64).max(0.05);

                    // 确保最小缩放比例不小于选区大小
                    let img_width = img.width() as f64;
                    let img_height = img.height() as f64;
                    let min_scale =
                        (self.selection_size / img_width).max(self.selection_size / img_height);
                    let new_scale = new_scale.max(min_scale);

                    // 以鼠标为缩放中心进行缩放
                    let scale_ratio = new_scale / self.scale;
                    let new_x = mouse_x - canvas.get_bounding_client_rect().left() - self.x;
                    let new_y = mouse_y - canvas.get_bounding_client_rect().top() - self.y;

                    self.x =
                        mouse_x - new_x * scale_ratio - canvas.get_bounding_client_rect().left();
                    self.y =
                        mouse_y - new_y * scale_ratio - canvas.get_bounding_client_rect().top();
                    self.scale = new_scale;

                    // 限制图片位置保持在选区内部并适应缩放
                    let result = self.adjust_image_position(img, &canvas);
                    self.x = result.0;
                    self.y = result.1;
                    self.redraw();
                }
                false
            }
            Msg::MouseDown(event) => {
                event.stop_propagation();
                self.dragging = true;
                self.start_x = event.client_x() as f64;
                self.start_y = event.client_y() as f64;
                true
            }
            Msg::MouseUp(_) => {
                self.dragging = false;
                true
            }
            Msg::MouseMove(event) => {
                event.stop_propagation();
                if self.dragging {
                    let dx = event.client_x() as f64 - self.start_x;
                    let dy = event.client_y() as f64 - self.start_y;
                    self.x += dx;
                    self.y += dy;

                    if let Some(img) = &self.img {
                        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();

                        // 限制图片位置保持在选区内部
                        let result = self.adjust_image_position(img, &canvas);
                        self.x = result.0;
                        self.y = result.1;
                    }

                    self.start_x = event.client_x() as f64;
                    self.start_y = event.client_y() as f64;
                    self.redraw();
                }

                false
            }
            Msg::SubmitSelection => {
                if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
                    let context = canvas
                        .get_context("2d")
                        .unwrap()
                        .unwrap()
                        .dyn_into::<web_sys::CanvasRenderingContext2d>()
                        .unwrap();

                    // 提取选区图像数据
                    let selection_x = (canvas.width() as f64 - self.selection_size) / 2.0;
                    let selection_y = (canvas.height() as f64 - self.selection_size) / 2.0;
                    let data = context
                        .get_image_data(
                            selection_x,
                            selection_y,
                            self.selection_size,
                            self.selection_size,
                        )
                        .unwrap();

                    match &ctx.props().submit {
                        SubmitOption::ImageData(callback) => callback.emit(data),
                        SubmitOption::DataUrl(callback) => {
                            callback.emit(self.image_data_to_url(&data))
                        }
                        SubmitOption::Data(callback) => callback.emit(data.data().0),
                    }
                }
                false
            }
            Msg::ImageLoaded => {
                if let Some(img) = &self.img {
                    if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
                        let image_width = img.width() as f64;
                        let image_height = img.height() as f64;
                        let canvas_width = canvas.width() as f64;
                        let canvas_height = canvas.height() as f64;

                        // 宽高比
                        let scale_height = canvas_height / image_height;
                        let scale_width = if image_height <= self.selection_size {
                            1.0
                        } else {
                            canvas_width / image_width
                        };

                        // 使用canvas高度调整比例，确保能看到整个选区
                        let scale = scale_height
                            .min(scale_width)
                            .max(self.selection_size / image_height);

                        self.scale = scale;
                        self.x = (canvas_width - image_width * scale) / 2.0;
                        self.y = (canvas_height - image_height * scale) / 2.0;
                    }
                }
                self.redraw();
                false
            }
            Msg::TouchStart(event) => {
                event.stop_propagation();
                self.update_touches(&event);
                false
            }
            Msg::TouchMove(event) => {
                event.stop_propagation();
                event.prevent_default();
                self.handle_touch_move(&event);
                self.update_touches(&event);
                false
            }
            Msg::TouchEnd(event) => {
                event.stop_propagation();
                event.prevent_default();
                self.update_touches(&event);
                false
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
                canvas.set_width(canvas.parent_element().unwrap().client_width() as u32);
                canvas.set_height(canvas.parent_element().unwrap().client_height() as u32);
            }
            if let Some(ref url) = ctx.props().avatar_url {
                self.load_img(ctx, url);
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_wheel = ctx.link().callback(|e: web_sys::WheelEvent| Msg::Wheel(e));
        let on_mousedown = ctx
            .link()
            .callback(|e: web_sys::MouseEvent| Msg::MouseDown(e));
        let on_mouseup = ctx
            .link()
            .callback(|e: web_sys::MouseEvent| Msg::MouseUp(e));
        let on_mousemove = ctx
            .link()
            .callback(|e: web_sys::MouseEvent| Msg::MouseMove(e));
        let ontouchstart = ctx.link().callback(Msg::TouchStart);
        let ontouchmove = ctx.link().callback(Msg::TouchMove);
        let ontouchend = ctx.link().callback(Msg::TouchEnd);
        let on_submit = ctx.link().callback(|_| Msg::SubmitSelection);
        html! {
            <div style="display: flex;
                        flex-direction: column;
                        width: 100%;
                        height:100%;
                        background-color: white;">
                <div style="position: absolute;
                            top: 0;
                            left: 0;
                            width: 100%;
                            padding: .5rem;
                            display: flex;
                            justify-content: space-between;
                            align-items: center;">
                    <label for="avatar-setter"
                        style="width: 5rem;
                            height: 2rem;
                            line-height: 2rem;
                            text-align: center;
                            background-color: #fefefe;
                            border-radius: .3rem;">
                        { &ctx.props().choose_text }
                        <input id="avatar-setter"
                            type="file"
                            accept="image/*"
                            hidden=true
                            multiple=false
                            onchange={ctx.link().callback(Msg::Files)}/>
                    </label>
                    <div
                        style="width: 5rem; height: 2rem; line-height: 2rem; text-align: center; background-color: green; color: white; border-radius: .3rem;"
                        onclick={on_submit}>
                        { &ctx.props().submit_text }
                    </div>
                    <div
                        style="width: 5rem; height: 2rem; line-height: 2rem; text-align: center; background-color: white; border-radius: .3rem;"
                        onclick={ctx.props().close.reform(|_|{})}>
                        { &ctx.props().cancel_text }
                    </div>
                </div>
                <canvas
                    style="width: 100%; height: 100%;"
                    ref={self.canvas_ref.clone()}
                    onwheel={on_wheel}
                    onmousedown={on_mousedown}
                    onmouseup={on_mouseup}
                    onmousemove={on_mousemove}
                    {ontouchstart}
                    {ontouchmove}
                    {ontouchend}
                >
                </canvas>
            </div>
        }
    }
}

impl Avatar {
    fn load_img(&mut self, ctx: &Context<Self>, url: &str) {
        let img = HtmlImageElement::new().unwrap();
        img.set_src(url);
        let ctx = ctx.link().clone();
        let closure = Closure::wrap(Box::new(move || {
            ctx.send_message(Msg::ImageLoaded); // 通知重绘
        }) as Box<dyn FnMut()>);
        img.set_onload(Some(closure.as_ref().unchecked_ref()));
        self.img = Some(img);
        self.avatar_onload = Some(closure);
    }

    fn adjust_image_position(
        &self,
        img: &HtmlImageElement,
        canvas: &HtmlCanvasElement,
    ) -> (f64, f64) {
        let selection_x = (canvas.width() as f64 - self.selection_size) / 2.0;
        let selection_y = (canvas.height() as f64 - self.selection_size) / 2.0;

        // 限制图片位置保持在选区内部并调整图片位置
        let img_width = img.width() as f64;
        let img_height = img.height() as f64;
        let min_x = selection_x - img_width * self.scale + self.selection_size;
        let max_x = selection_x;
        let min_y = selection_y - img_height * self.scale + self.selection_size;
        let max_y = selection_y;
        (self.x.min(max_x).max(min_x), self.y.min(max_y).max(min_y))
    }

    fn update_touches(&mut self, event: &TouchEvent) {
        self.touches.clear();
        for i in 0..event.touches().length() {
            if let Some(touch) = event.touches().item(i) {
                self.touches.insert(
                    touch.identifier(),
                    (touch.client_x() as f64, touch.client_y() as f64),
                );
            }
        }
    }

    fn handle_touch_move(&mut self, event: &TouchEvent) {
        let touches: Vec<_> = self.touches.values().cloned().collect();
        if touches.len() == 2 {
            let new_touches: Vec<_> = (0..event.touches().length())
                .map(|i| {
                    let touch = event.touches().item(i).unwrap();
                    (touch.client_x() as f64, touch.client_y() as f64)
                })
                .collect();

            let old_distance = ((touches[0].0 - touches[1].0).powi(2)
                + (touches[0].1 - touches[1].1).powi(2))
            .sqrt();
            let new_distance = ((new_touches[0].0 - new_touches[1].0).powi(2)
                + (new_touches[1].1 - new_touches[0].1).powi(2))
            .sqrt();

            let delta = new_distance - old_distance;
            if delta.abs() > 1.0 {
                // 增加一个基本的阈值，避免误判
                self.zoom(
                    delta,
                    (new_touches[0].0 + new_touches[1].0) / 2.0,
                    (new_touches[0].1 + new_touches[1].1) / 2.0,
                );
            }
        } else if touches.len() == 1 {
            if let Some((sx, sy)) = self.touches.values().next() {
                if let Some(touch) = event.touches().item(0) {
                    let dx = touch.client_x() as f64 - sx;
                    let dy = touch.client_y() as f64 - sy;

                    self.x += dx;
                    self.y += dy;

                    if let Some(img) = &self.img {
                        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
                        let result = self.adjust_image_position(img, &canvas);
                        self.x = result.0;
                        self.y = result.1;
                    }

                    self.redraw();
                }
            }
        }
    }

    fn zoom(&mut self, delta: f64, center_x: f64, center_y: f64) {
        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
        if let Some(img) = &self.img {
            let factor = 0.01; // 移动端缩放灵敏度，调整值避免过于灵敏
            let delta = delta * factor;
            let new_scale = (self.scale + delta).max(0.1);

            let img_width = img.width() as f64;
            let img_height = img.height() as f64;
            let min_scale = (self.selection_size / img_width).max(self.selection_size / img_height);
            let new_scale = new_scale.max(min_scale);

            let scale_ratio = new_scale / self.scale;
            let new_x = center_x - canvas.get_bounding_client_rect().left() - self.x;
            let new_y = center_y - canvas.get_bounding_client_rect().top() - self.y;

            self.x = center_x - new_x * scale_ratio - canvas.get_bounding_client_rect().left();
            self.y = center_y - new_y * scale_ratio - canvas.get_bounding_client_rect().top();
            self.scale = new_scale;

            let result = self.adjust_image_position(img, &canvas);
            self.x = result.0;
            self.y = result.1;
            self.redraw();
        }
    }
    fn redraw(&self) {
        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        log::info!(
            "redraw; canvas width: {}, {}, {}",
            canvas.client_width(),
            canvas.height(),
            canvas.client_height()
        );
        if let Some(img) = &self.img {
            context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
            context.save();
            context.translate(self.x, self.y).unwrap();
            context.scale(self.scale, self.scale).unwrap();
            context
                .draw_image_with_html_image_element(img, 0.0, 0.0)
                .unwrap();
            context.restore();

            // 绘制半透明遮盖层
            context.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.5)"));
            let overlay_width = canvas.width() as f64;
            let overlay_height = canvas.height() as f64;
            let selection_size = self.selection_size;
            let selection_x = if overlay_width < selection_size {
                0.
            } else {
                (overlay_width - selection_size) / 2.0
            };
            let selection_y = if overlay_height < selection_size {
                0.
            } else {
                (overlay_height - selection_size) / 2.0
            };

            // 上边的遮盖层
            context.fill_rect(0.0, 0.0, overlay_width, selection_y);
            // 下边的遮盖层
            context.fill_rect(
                0.0,
                selection_y + selection_size,
                overlay_width,
                overlay_height - (selection_y + selection_size),
            );
            // 左边的遮盖层
            context.fill_rect(0.0, selection_y, selection_x, selection_size);
            // 右边的遮盖层
            context.fill_rect(
                selection_x + selection_size,
                selection_y,
                overlay_width - (selection_x + selection_size),
                selection_size,
            );
        }
    }

    fn image_data_to_url(&self, image_data: &ImageData) -> String {
        let canvas = document().create_element("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();
        canvas.set_width(image_data.width());
        canvas.set_height(image_data.height());
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        ctx.put_image_data(image_data, 0.0, 0.0).unwrap();
        canvas.to_data_url().unwrap()
    }
}
