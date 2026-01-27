use std::alloc::Allocator;

pub struct VglAllocator;

unsafe extern "C" {
    pub fn vglAlloc(size: u32, type_: i32) -> *mut std::ffi::c_void;
    pub fn vglFree(addr: *mut std::ffi::c_void);
}

unsafe impl Allocator for VglAllocator {
    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        unsafe {
            Ok(std::ptr::NonNull::slice_from_raw_parts(
                std::ptr::NonNull::new(vglAlloc(layout.size() as u32, 1))
                    .ok_or(std::alloc::AllocError)?
                    .cast(),
                layout.size(),
            ))
        }
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        unsafe { vglFree(ptr.as_ptr() as _) }
    }
}
