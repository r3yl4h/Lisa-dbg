use std::alloc::{alloc, Layout};

pub fn alloc_size_align<T: Sized>() -> Result<*mut T, anyhow::Error>{
    let layout = Layout::from_size_align(size_of::<T>(), align_of::<T>())?;
    Ok(unsafe {alloc(layout) as *mut T})
}