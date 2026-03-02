#![allow(warnings)]

// Options used:
//   * runtime_path: "wit_bindgen_rt"
#[repr(u8)]
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    /// Invalid payload
    Invalid,
    /// Payload too large
    TooLarge,
    /// Not supported
    Unsupported,
    /// Not enough space, similar to `ENOMEM` in POSIX.
    InsufficientMemory,
    /// No space left on device, similar to `ENOSPC` in POSIX.
    InsufficientSpace,
    /// Operation not permitted, similar to `EPERM` in POSIX.
    NotPermitted,
    /// An unknown error occurred.
    Unknown,
}
impl ::core::fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            ErrorKind::Invalid => f.debug_tuple("ErrorKind::Invalid").finish(),
            ErrorKind::TooLarge => f.debug_tuple("ErrorKind::TooLarge").finish(),
            ErrorKind::Unsupported => f.debug_tuple("ErrorKind::Unsupported").finish(),
            ErrorKind::InsufficientMemory => {
                f.debug_tuple("ErrorKind::InsufficientMemory").finish()
            }
            ErrorKind::InsufficientSpace => f.debug_tuple("ErrorKind::InsufficientSpace").finish(),
            ErrorKind::NotPermitted => f.debug_tuple("ErrorKind::NotPermitted").finish(),
            ErrorKind::Unknown => f.debug_tuple("ErrorKind::Unknown").finish(),
        }
    }
}
impl ErrorKind {
    #[doc(hidden)]
    pub unsafe fn _lift(val: u8) -> ErrorKind {
        if !cfg!(debug_assertions) {
            return ::core::mem::transmute(val);
        }
        match val {
            0 => ErrorKind::Invalid,
            1 => ErrorKind::TooLarge,
            2 => ErrorKind::Unsupported,
            3 => ErrorKind::InsufficientMemory,
            4 => ErrorKind::InsufficientSpace,
            5 => ErrorKind::NotPermitted,
            6 => ErrorKind::Unknown,
            _ => panic!("invalid enum discriminant"),
        }
    }
}
#[derive(Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: _rt::String,
}
impl ::core::fmt::Debug for Error {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("message", &self.message)
            .finish()
    }
}
impl ::core::fmt::Display for Error {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_execute_cabi<T: Tilt>(arg0: *mut u8, arg1: usize) -> *mut u8 {
    #[cfg(target_arch = "wasm32")]
    _rt::run_ctors_once();
    let len0 = arg1;
    let result1 = T::execute(_rt::Vec::from_raw_parts(arg0.cast(), len0, len0));
    let ptr2 = (&raw mut _RET_AREA.0).cast::<u8>();
    match result1 {
        Ok(e) => {
            *ptr2.add(0).cast::<u8>() = (0i32) as u8;
            let vec3 = (e).into_boxed_slice();
            let ptr3 = vec3.as_ptr().cast::<u8>();
            let len3 = vec3.len();
            ::core::mem::forget(vec3);
            *ptr2
                .add(2 * ::core::mem::size_of::<*const u8>())
                .cast::<usize>() = len3;
            *ptr2
                .add(::core::mem::size_of::<*const u8>())
                .cast::<*mut u8>() = ptr3.cast_mut();
        }
        Err(e) => {
            *ptr2.add(0).cast::<u8>() = (1i32) as u8;
            let Error {
                kind: kind4,
                message: message4,
            } = e;
            *ptr2.add(::core::mem::size_of::<*const u8>()).cast::<u8>() =
                (kind4.clone() as i32) as u8;
            let vec5 = (message4.into_bytes()).into_boxed_slice();
            let ptr5 = vec5.as_ptr().cast::<u8>();
            let len5 = vec5.len();
            ::core::mem::forget(vec5);
            *ptr2
                .add(3 * ::core::mem::size_of::<*const u8>())
                .cast::<usize>() = len5;
            *ptr2
                .add(2 * ::core::mem::size_of::<*const u8>())
                .cast::<*mut u8>() = ptr5.cast_mut();
        }
    };
    ptr2
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_execute<T: Tilt>(arg0: *mut u8) {
    let l0 = i32::from(*arg0.add(0).cast::<u8>());
    match l0 {
        0 => {
            let l1 = *arg0
                .add(::core::mem::size_of::<*const u8>())
                .cast::<*mut u8>();
            let l2 = *arg0
                .add(2 * ::core::mem::size_of::<*const u8>())
                .cast::<usize>();
            let base3 = l1;
            let len3 = l2;
            _rt::cabi_dealloc(base3, len3 * 1, 1);
        }
        _ => {
            let l4 = *arg0
                .add(2 * ::core::mem::size_of::<*const u8>())
                .cast::<*mut u8>();
            let l5 = *arg0
                .add(3 * ::core::mem::size_of::<*const u8>())
                .cast::<usize>();
            _rt::cabi_dealloc(l4, l5, 1);
        }
    }
}
pub trait Tilt {
    fn execute(req: _rt::Vec<u8>) -> Result<_rt::Vec<u8>, Error>;
}
#[doc(hidden)]
macro_rules! __export_world_tilt_cabi {
    ($ty:ident with_types_in $($path_to_types:tt)*) => {
        const _ : () = { #[unsafe (export_name = "execute")] unsafe extern "C" fn
        export_execute(arg0 : * mut u8, arg1 : usize,) -> * mut u8 { unsafe {
        $($path_to_types)*:: _export_execute_cabi::<$ty > (arg0, arg1) } } #[unsafe
        (export_name = "cabi_post_execute")] unsafe extern "C" fn
        _post_return_execute(arg0 : * mut u8,) { unsafe { $($path_to_types)*::
        __post_return_execute::<$ty > (arg0) } } };
    };
}
#[doc(hidden)]
pub(crate) use __export_world_tilt_cabi;
#[cfg_attr(target_pointer_width = "64", repr(align(8)))]
#[cfg_attr(target_pointer_width = "32", repr(align(4)))]
struct _RetArea([::core::mem::MaybeUninit<u8>; 4 * ::core::mem::size_of::<*const u8>()]);
static mut _RET_AREA: _RetArea =
    _RetArea([::core::mem::MaybeUninit::uninit(); 4 * ::core::mem::size_of::<*const u8>()]);
#[rustfmt::skip]
mod _rt {
    #![allow(dead_code, clippy::all)]
    pub use alloc_crate::string::String;
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
    }
    pub use alloc_crate::vec::Vec;
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    extern crate alloc as alloc_crate;
    pub use alloc_crate::alloc;
}
/// Generates `#[unsafe(no_mangle)]` functions to export the specified type as
/// the root implementation of all generated traits.
///
/// For more information see the documentation of `wit_bindgen::generate!`.
///
/// ```rust
/// # macro_rules! export{ ($($t:tt)*) => (); }
/// # trait Tilt {}
/// struct MyType;
///
/// impl Tilt for MyType {
///     // ...
/// }
///
/// export!(MyType);
/// ```
#[allow(unused_macros)]
#[doc(hidden)]
macro_rules! __export_tilt_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*:: __export_world_tilt_cabi!($ty with_types_in
        $($path_to_types_root)*);
    };
}
#[doc(inline)]
pub(crate) use __export_tilt_impl as export;
#[cfg(target_arch = "wasm32")]
#[unsafe(link_section = "component-type:wit-bindgen:0.41.0:tilt:app@0.1.0:tilt:encoded world")]
#[doc(hidden)]
#[allow(clippy::octal_escapes)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 323] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xc8\x01\x01A\x02\x01\
A\x08\x01m\x07\x07invalid\x09too-large\x0bunsupported\x13insufficient-memory\x12\
insufficient-space\x0dnot-permitted\x07unknown\x03\0\x0aerror-kind\x03\0\0\x01r\x02\
\x04kind\x01\x07messages\x03\0\x05error\x03\0\x02\x01p}\x01j\x01\x04\x01\x03\x01\
@\x01\x03req\x04\0\x05\x04\0\x07execute\x01\x06\x04\0\x13tilt:app/tilt@0.1.0\x04\
\0\x0b\x0a\x01\0\x04tilt\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-\
component\x070.227.1\x10wit-bindgen-rust\x060.41.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
