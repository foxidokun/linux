use kernel::bindings;
use kernel::prelude::*;

use pin_init::pin_init_from_closure;

#[repr(transparent)]
pub struct KeyEntry(bindings::key_entry);

impl KeyEntry {
    pub const fn new_key(code: u32, keycode: u16) -> Self {
        Self(bindings::key_entry {
            type_: bindings::KE_KEY as i32,
            code,
            __bindgen_anon_1: bindings::key_entry__bindgen_ty_1 {
                keycode
            },
        })
    }

    pub const fn terminating() -> Self {
        Self(bindings::key_entry {
            type_: bindings::KE_END as i32,
            ..pin_init::zeroed()
        })
    }
}

// INVARIANT: SparseKeymap is always initialized
#[pin_data(PinnedDrop)]
pub struct SparseKeymap {
    dev: *mut bindings::input_dev,
}

// It's Send, but not Sync
unsafe impl Send for SparseKeymap {}

impl SparseKeymap {
    pub fn new<'a>(
        name: &'static CStr,
        phys: &'static CStr,
        keymap: &'a [KeyEntry],
    ) -> impl PinInit<Self, Error> + use<'a> {
        unsafe {
            pin_init_from_closure(move |this: *mut Self| {
                (*this).dev = bindings::input_allocate_device();

                (*(*this).dev).name = name.as_char_ptr();
                (*(*this).dev).phys = phys.as_char_ptr();

                let keymap = ::core::mem::transmute(keymap.as_ptr());

                bindings::sparse_keymap_setup((*this).dev, keymap, None);

                bindings::input_register_device((*this).dev);

                Ok(())
            })
        }
    }

    pub fn report_event(&mut self, code: u32) {
        unsafe {
            bindings::sparse_keymap_report_event(self.dev, code, 1, true);
        }
    }
}

#[pinned_drop]
impl PinnedDrop for SparseKeymap {
    fn drop(self: Pin<&mut Self>) {
        unsafe {
            bindings::input_unregister_device(self.dev as *mut _);
        }
    }
}
