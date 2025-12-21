//! This is SAMPLE DRIVER
//! It doesn't aim into upstream and serves only demonstration purposes for
//! RFC patchset

use kernel::acpi::AcpiBuffer;
use kernel::bindings;
use kernel::new_mutex;
use kernel::sparse_keymap::{KeyEntry, SparseKeymap};
use kernel::sync::atomic::{Atomic, Relaxed};
use kernel::sync::Mutex;

use kernel::{
    acpi::AcpiObject,
    device, module_wmi_driver, pr_info,
    prelude::*,
    wmi::{self, Device, DeviceId, Driver},
    wmi_device_table,
};

wmi_device_table!(
    REDMI_TABLE,
    MODULE_REDMI_TABLE,
    <RedmiWMIDriver as Driver>::IdInfo,
    [(DeviceId::new(b"46C93E13-EE9B-4262-8488-563BCA757FEF"), 12)]
);

#[pin_data]
struct RedmiWMIDriver {
    #[pin]
    dev: Mutex<SparseKeymap>,
}

const REDMI_KEYMAP: [KeyEntry; 2] = [
    KeyEntry::new_key(0x00000201, bindings::KEY_SELECTIVE_SCREENSHOT as u16),
    KeyEntry::terminating(),
];

#[vtable]
impl wmi::Driver for RedmiWMIDriver {
    type IdInfo = i32;

    const TABLE: &'static dyn kernel::device_id::IdTable<DeviceId, Self::IdInfo> = &REDMI_TABLE;

    fn probe(
        _: &Device<kernel::device::Core>,
        id_info: &Self::IdInfo,
    ) -> impl PinInit<Self, Error> {
        pr_err!("Rust WMI Sample Driver probed with val\n");

        let sp_keymap_init = SparseKeymap::new(c"Redmibook WMI keys", c"wmi/input0", &REDMI_KEYMAP);

        try_pin_init!(
            Self {
                dev <- new_mutex!(sp_keymap_init)
            }
        ? Error)
    }

    fn notify(self: Pin<&Self>, dev: &Device<device::Core>, obj: Option<&AcpiObject>) {
        pr_err!("Rust WMI Sample Driver notified!");

        if let Err(err) = self.notify_inner(obj) {
            dev_err!(dev.as_ref(), "Failed to process WMI notification with err: '{}'", err);
        }
    }
}

impl RedmiWMIDriver {
    fn notify_inner(self: Pin<&Self>, obj: Option<&AcpiObject>) -> Result<(), &'static str> {
        let obj = obj.ok_or("Missing ACPI payload")?;
        let obj: &AcpiBuffer = obj.try_into().map_err(|_| "Wrong ACPI payload type")?;
        if obj.len() < 32 {
            return Err("Incorrect buffer length");
        }

        let val = u32::from_le_bytes(obj[0..4].try_into().unwrap());

        let mut inp = self.dev.lock();
        
        inp.report_event(val);

        Ok(())
    }
}

module_wmi_driver!(
    type: RedmiWMIDriver,
    name: "redmi_wmi_sample",
    authors: ["Gladyshev Ilya"],
    description: "SAMPLE DRIVER for RFC demonstration only",
    license: "GPL v2",
);
