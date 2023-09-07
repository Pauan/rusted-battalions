use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use raw_window_handle::{
    HasRawWindowHandle, HasRawDisplayHandle, RawWindowHandle,
    RawDisplayHandle, WebWindowHandle, WebDisplayHandle,
};
use std::sync::atomic::{AtomicU32, Ordering};


/// Renders into an HTML <canvas>
pub struct Window {
    id: u32,
    canvas: HtmlCanvasElement,
}

impl Window {
    pub fn new() -> Self {
        static ID: AtomicU32 = AtomicU32::new(1);

        let id = ID.fetch_add(1, Ordering::SeqCst);

        let canvas = web_sys::window().unwrap()
            .document().unwrap()
            .create_element("canvas").unwrap()
            .unchecked_into::<HtmlCanvasElement>();

        canvas.set_attribute("data-raw-handle", &id.to_string()).unwrap();

        Self { id, canvas }
    }

    pub fn canvas(&self) -> HtmlCanvasElement {
        self.canvas.clone()
    }
}

/// SAFETY: This is safe because each Window has a guaranteed unique ID which is greater than 0
unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = WebWindowHandle::empty();
        handle.id = self.id;
        RawWindowHandle::Web(handle)
    }
}

/// SAFETY: RawDisplayHandle::Web is always safe.
unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        let handle = WebDisplayHandle::empty();
        RawDisplayHandle::Web(handle)
    }
}
