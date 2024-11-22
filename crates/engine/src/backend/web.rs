use raw_window_handle::{
    HasWindowHandle, HasDisplayHandle, WindowHandle,
    DisplayHandle, RawWindowHandle, WebWindowHandle, HandleError,
};
use std::sync::atomic::{AtomicU32, Ordering};


/// Renders into an HTML <canvas>
pub struct Window {
    id: u32,
}

impl Window {
    pub fn new() -> Self {
        static ID: AtomicU32 = AtomicU32::new(1);

        let id = ID.fetch_add(1, Ordering::SeqCst);

        Self { id }
    }

    /// The unique ID for this [`Window`].
    ///
    /// You should assign this to the `data-raw-handle` attribute on a `<canvas>`
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle, HandleError> {
        // SAFETY: This is safe because we guarantee that each ID is unique
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::from(WebWindowHandle::new(self.id))) })
    }
}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<DisplayHandle, HandleError> {
        Ok(DisplayHandle::web())
    }
}
