use libc::{c_void, calloc, free, malloc};
use std::{mem::size_of, ptr::copy_nonoverlapping};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
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
pub extern "C" fn new_app(term: &mut Term, args: *const Term, length: usize) {
    term.args = alloc_terms(length);
    unsafe { copy_nonoverlapping(args, term.args, length) };
}

#[no_mangle]
pub extern "C" fn new_partial(term: &mut Term, args: *const Term, length: usize) {
    term.args = calloc_terms(term.capacity as usize);
    unsafe { copy_nonoverlapping(args, term.args, length) };

    let mut fun = *term;
    fun.length = 0;

    let last = term.capacity - 1;
    *arg_mut(term, last as usize) = fun;

    term.fun = noop;
    term.length = length as u16;
}

#[no_mangle]
pub extern "C" fn apply_partial(term: &mut Term, args: *const Term, length: usize) {
    let last = term.capacity - 1;
    let fun = *arg_mut(term, last as usize);

    let offset = term.length;
    unsafe { copy_nonoverlapping(args, arg_mut(term, offset as usize), length) };

    term.length += length as u16;

    if term.length == term.capacity {
        term.fun = fun.fun;
    }
}

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
    dealloc_terms(term.args);
}

#[no_mangle]
pub extern "C" fn free_term(term: &mut Term) {
    for i in 0..term.length as usize {
        free_term(arg_mut(term, i));
    }
    free_args(term);
}

fn as_ref<'a>(term: *const Term) -> &'a Term {
    unsafe { term.as_ref().unwrap_unchecked() }
}

fn as_mut<'a>(term: *mut Term) -> &'a mut Term {
    unsafe { term.as_mut().unwrap_unchecked() }
}

fn arg(term: &Term, i: usize) -> &Term {
    let arg = unsafe { term.args.add(i) };
    as_ref(arg)
}

fn arg_mut(term: &mut Term, i: usize) -> &mut Term {
    let arg = unsafe { term.args.add(i) };
    as_mut(arg)
}

fn alloc_terms(capacity: usize) -> *mut Term {
    let size = capacity * size_of::<Term>();
    (unsafe { malloc(size) } as *mut Term)
}

fn calloc_terms(capacity: usize) -> *mut Term {
    let size = size_of::<Term>();
    (unsafe { calloc(capacity, size) } as *mut Term)
}

fn dealloc_terms(terms: *mut Term) {
    unsafe { free(terms as *mut c_void) };
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ptr::null_mut;

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

    #[test]
    fn test_free_term() {
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

        assert_eq!(arg(&term1, 0).symbol, 2);
        assert_eq!(arg(&term1, 1).symbol, 2);

        free_term(&mut term1);
    }

    #[test]
    fn test_copy_nil() {
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

        assert_eq!(term2.symbol, 1);

        free_term(&mut term1);
        free_term(&mut term2);
    }

    #[test]
    fn test_copy() {
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

        assert_eq!(term3.symbol, 1);
        assert_eq!(arg(&term3, 0).symbol, 2);
        assert_eq!(arg(&term3, 1).symbol, 2);

        free_term(&mut term1);
        free_term(&mut term3);
    }

    #[test]
    fn test_new_app() {
        let mut term1 = Term {
            fun: noop,
            args: null_mut(),
            symbol: 1,
            length: 1,
            capacity: 1,
        };

        let term2 = Term {
            fun: noop,
            args: null_mut(),
            symbol: 2,
            length: 0,
            capacity: 0,
        };

        let args = [term2];
        let length = args.len();
        new_app(&mut term1, args.as_ptr(), length);

        assert_eq!(term1.symbol, 1);
        assert_eq!(arg(&term1, 0).symbol, 2);

        free_term(&mut term1);
    }

    #[test]
    fn test_new_partial() {
        let mut term1 = Term {
            fun: noop,
            args: null_mut(),
            symbol: 1,
            length: 0,
            capacity: 2,
        };

        let term2 = Term {
            fun: noop,
            args: null_mut(),
            symbol: 2,
            length: 0,
            capacity: 0,
        };

        let args = [term2];
        let length = args.len();
        new_partial(&mut term1, args.as_ptr(), length);

        assert_eq!(term1.symbol, 1);
        assert_eq!(arg(&term1, 0).symbol, 2);
        assert_eq!(arg(&term1, 1).symbol, 1);

        free_term(&mut term1);
    }

    #[test]
    fn test_copy_partial() {
        // TODO
    }
}
