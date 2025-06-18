#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub struct RawVoidPtr(pub *const core::ffi::c_void);

impl Default for RawVoidPtr {
    fn default() -> Self {
        Self(core::ptr::null())
    }
}

unsafe impl Send for RawVoidPtr {}
unsafe impl Sync for RawVoidPtr {}
