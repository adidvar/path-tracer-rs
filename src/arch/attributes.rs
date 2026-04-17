use winit::{
    dpi::LogicalSize,
    window::{Window, WindowAttributes},
};

pub fn window_attributes(window_size: LogicalSize<u32>, window_title: &str) -> WindowAttributes {
    let mut attributes = Window::default_attributes()
        .with_title(window_title)
        .with_maximized(true);

    #[cfg(not(target_arch = "wasm32"))]
    {
        attributes = attributes.with_inner_size(window_size);
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = window_size;

        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id("wgpu-canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        attributes = attributes.with_canvas(Some(canvas));
    }

    attributes
}
