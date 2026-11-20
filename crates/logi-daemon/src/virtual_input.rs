use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

// Linux input subsystem constants
const EV_SYN: u16 = 0x00;
const SYN_REPORT: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;

const REL_X: u16 = 0x00;
const REL_Y: u16 = 0x01;
const REL_WHEEL: u16 = 0x08;
const REL_HWHEEL: u16 = 0x06;

// ioctl codes for /dev/uinput
const UI_SET_EVBIT: libc::c_ulong = 0x40045564;
const UI_SET_KEYBIT: libc::c_ulong = 0x40045565;
const UI_SET_RELBIT: libc::c_ulong = 0x40045566;
const UI_DEV_SETUP: libc::c_ulong = 0x405c5503;
const UI_DEV_CREATE: libc::c_ulong = 0x5501;
const UI_DEV_DESTROY: libc::c_ulong = 0x5502;

#[repr(C)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

#[repr(C)]
struct UinputSetup {
    id: InputId,
    name: [u8; 80],
    ff_effects_max: u32,
}

#[repr(C)]
struct InputEvent {
    time: libc::timeval,
    type_: u16,
    code: u16,
    value: i32,
}

pub struct VirtualInputDevice {
    file: File,
}

impl VirtualInputDevice {
    pub fn new(device_name: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open("/dev/uinput")?;

        let fd = file.as_raw_fd();

        unsafe {
            if libc::ioctl(fd, UI_SET_EVBIT, EV_KEY as libc::c_int) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::ioctl(fd, UI_SET_EVBIT, EV_REL as libc::c_int) < 0 {
                return Err(std::io::Error::last_os_error());
            }

            for rel in [REL_X, REL_Y, REL_WHEEL, REL_HWHEEL] {
                libc::ioctl(fd, UI_SET_RELBIT, rel as libc::c_int);
            }

            for key in 1..255 {
                libc::ioctl(fd, UI_SET_KEYBIT, key as libc::c_int);
            }
            for btn in 0x110..0x118 {
                libc::ioctl(fd, UI_SET_KEYBIT, btn as libc::c_int);
            }

            let mut setup: UinputSetup = std::mem::zeroed();
            setup.id.bustype = 0x03; // BUS_USB
            setup.id.vendor = 0x046d;
            setup.id.product = 0x9999;
            setup.id.version = 1;

            let name_bytes = device_name.as_bytes();
            let copy_len = name_bytes.len().min(79);
            setup.name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

            if libc::ioctl(fd, UI_DEV_SETUP, &setup) < 0 {
                return Err(std::io::Error::last_os_error());
            }

            if libc::ioctl(fd, UI_DEV_CREATE) < 0 {
                return Err(std::io::Error::last_os_error());
            }
        }

        Ok(Self { file })
    }

    fn write_event(&mut self, type_: u16, code: u16, value: i32) -> Result<(), std::io::Error> {
        let event = InputEvent {
            time: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_,
            code,
            value,
        };

        let slice = unsafe {
            std::slice::from_raw_parts(
                &event as *const InputEvent as *const u8,
                std::mem::size_of::<InputEvent>(),
            )
        };

        self.file.write_all(slice)?;
        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), std::io::Error> {
        self.write_event(EV_SYN, SYN_REPORT, 0)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn emit_key(&mut self, key_code: u16, pressed: bool) -> Result<(), std::io::Error> {
        self.write_event(EV_KEY, key_code, if pressed { 1 } else { 0 })?;
        self.sync()
    }

    pub async fn emit_click(&mut self, key_code: u16) -> Result<(), std::io::Error> {
        self.emit_key(key_code, true)?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.emit_key(key_code, false)?;
        Ok(())
    }

    pub async fn emit_combo(&mut self, keys: &[u16]) -> Result<(), std::io::Error> {
        for &k in keys {
            self.write_event(EV_KEY, k, 1)?;
        }
        self.sync()?;

        // Hold modifier keys long enough for Wayland/X11 to register
        tokio::time::sleep(Duration::from_millis(25)).await;

        for &k in keys.iter().rev() {
            self.write_event(EV_KEY, k, 0)?;
        }
        self.sync()
    }

    pub fn emit_rel(&mut self, rel_code: u16, delta: i32) -> Result<(), std::io::Error> {
        self.write_event(EV_REL, rel_code, delta)?;
        self.sync()
    }
}

impl Drop for VirtualInputDevice {
    fn drop(&mut self) {
        let fd = self.file.as_raw_fd();
        unsafe {
            libc::ioctl(fd, UI_DEV_DESTROY);
        }
    }
}
