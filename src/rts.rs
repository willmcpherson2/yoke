use std::alloc::{alloc, dealloc, Layout};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Term {
    fun: extern "C" fn(*mut Term),
    args: *mut Term,
    symbol: u32,
    length: u16,
    capacity: u16,
}

#[no_mangle]
pub extern "C" fn noop(_term: *mut Term) {}

#[no_mangle]
pub extern "C" fn copy(dest: &mut Term, src: &Term) {
    *dest = *src;

    if src.capacity == 0 {
        return;
    }

    let size = src.capacity as usize;
    dest.args = alloc_terms(size);

    for i in 0..src.capacity as usize {
        copy(arg_mut(dest, i), arg(src, i));
    }
}

#[no_mangle]
pub extern "C" fn free_args(term: &mut Term) {
    dealloc_terms(term.args, term.capacity as usize);
}

#[no_mangle]
pub extern "C" fn free_term(term: &mut Term) {
    for i in 0..term.length as usize {
        free_term(arg_mut(term, i));
    }
    free_args(term);
}

fn as_ref<'a>(term: *const Term) -> &'a Term {
    unsafe { term.as_ref() }.unwrap()
}

fn as_mut<'a>(term: *mut Term) -> &'a mut Term {
    unsafe { term.as_mut() }.unwrap()
}

fn arg(term: &Term, i: usize) -> &Term {
    let arg = unsafe { term.args.add(i) };
    as_ref(arg)
}

fn arg_mut(term: &mut Term, i: usize) -> &mut Term {
    let arg = unsafe { term.args.add(i) };
    as_mut(arg)
}

fn terms_layout(capacity: usize) -> Layout {
    let size = std::mem::size_of::<Term>() * capacity;
    let align = std::mem::align_of::<Term>();
    Layout::from_size_align(size, align).unwrap()
}

fn alloc_terms(capacity: usize) -> *mut Term {
    let layout = terms_layout(capacity);
    (unsafe { alloc(layout) } as *mut Term)
}

fn dealloc_terms(terms: *mut Term, capacity: usize) {
    if terms.is_null() {
        return;
    }

    let layout = terms_layout(capacity);
    unsafe { dealloc(terms as *mut u8, layout) };
}

#[derive(Clone, Debug)]
struct ShowTerm {
    fun: usize,
    args_ptr: usize,
    args: Vec<ShowTerm>,
    symbol: u32,
    length: u16,
    capacity: u16,
}

fn show_term(term: &Term) -> ShowTerm {
    let mut args = vec![];
    for i in 0..term.length as usize {
        let arg = as_ref(arg(term, i));
        args.push(show_term(arg));
    }

    ShowTerm {
        fun: term.fun as usize,
        args_ptr: term.args as usize,
        args,
        symbol: term.symbol,
        length: term.length,
        capacity: term.capacity,
    }
}

#[cfg(test)]
mod test {
    use std::ptr::null_mut;

    use super::*;

    #[test]
    fn test_rts_free_term() {
        let mut term1 = Term {
            fun: noop,
            args: alloc_terms(2),
            symbol: 1,
            length: 2,
            capacity: 2,
        };

        let term2 = Term {
            fun: noop,
            args: null_mut(),
            symbol: 2,
            length: 0,
            capacity: 0,
        };

        *arg_mut(&mut term1, 0) = term2;
        *arg_mut(&mut term1, 1) = term2;

        free_term(&mut term1);
    }

    #[test]
    fn test_rts_copy_nil() {
        let mut term1 = Term {
            fun: noop,
            args: null_mut(),
            symbol: 1,
            length: 0,
            capacity: 0,
        };

        let mut term2 = Term {
            fun: noop,
            args: null_mut(),
            symbol: 2,
            length: 0,
            capacity: 0,
        };

        copy(&mut term2, &term1);

        free_term(&mut term1);
        free_term(&mut term2);
    }

    #[test]
    fn test_rts_copy() {
        let mut term1 = Term {
            fun: noop,
            args: alloc_terms(2),
            symbol: 1,
            length: 2,
            capacity: 2,
        };
        let term2 = Term {
            fun: noop,
            args: null_mut(),
            symbol: 2,
            length: 0,
            capacity: 0,
        };
        *arg_mut(&mut term1, 0) = term2;
        *arg_mut(&mut term1, 1) = term2;

        let mut term3 = Term {
            fun: noop,
            args: null_mut(),
            symbol: 3,
            length: 0,
            capacity: 0,
        };
        copy(&mut term3, &term1);

        free_term(&mut term1);
        free_term(&mut term3);
    }
}
