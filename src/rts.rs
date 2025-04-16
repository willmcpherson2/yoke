#[repr(C)]
pub struct Term {
    fun: fn(*mut Term),
    args: *mut Term,
    symbol: u32,
    length: u16,
    capacity: u16,
}

#[no_mangle]
pub extern "C" fn noop(_term: *mut Term) {}
