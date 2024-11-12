mod applications;
mod chaperone;
mod clientcore;
mod compositor;
mod convert;
mod input;
mod misc_unknown;
mod openxr_data;
mod overlay;
mod overlayview;
mod rendermodels;
mod screenshots;
mod system;
mod vulkan;

use clientcore::ClientCore;
use std::ffi::{c_char, c_void, CStr};
use std::sync::{Arc, Weak};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use bindings::vr;

impl Default for vr::ETrackingResult {
    fn default() -> Self {
        Self::TrackingResult_Uninitialized
    }
}

/// Types that are interfaces. Should only be implemented by generated code.
unsafe trait OpenVrInterface: 'static {
    type Vtable: Sync;
}

/// Trait for inheriting from an interface. Implemented by generated code.
/// The thread safety/usage patterns of OpenVR interfaces is not clear, so we err on the safe side and require
/// inheritors to be Sync.
unsafe trait Inherits<T: OpenVrInterface>: Sync
where
    Self: Sized,
{
    fn new_wrapped(wrapped: &Arc<Self>) -> VtableWrapper<T, Self>;
    fn init_fntable(init: &Arc<Self>) -> *mut c_void;
}

/// A wrapper around a vtable, to safely pass across FFI.
#[repr(C)]
struct VtableWrapper<T: OpenVrInterface, Wrapped> {
    base: T,
    wrapped: Weak<Wrapped>,
}

trait InterfaceImpl: Sync + Send + 'static {
    fn supported_versions() -> &'static [&'static CStr];
    /// Gets a specific interface version
    fn get_version(version: &CStr) -> Option<Box<dyn FnOnce(&Arc<Self>) -> *mut c_void>>;
}

macro_rules! warn_unimplemented {
    ($function:literal) => {
        crate::warn_once!(
            concat!("[ONCE] ", $function, " unimplemented ({}:{})"),
            file!(),
            line!()
        );
    };
}
use warn_unimplemented;
macro_rules! warn_once {
    ($($tt:tt)*) => {{
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            log::warn!($($tt)*);
        });
    }}
}
use warn_once;

fn init_logging() {
    static ONCE: std::sync::Once = std::sync::Once::new();

    ONCE.call_once(|| {
        let mut builder = env_logger::Builder::new();
        #[cfg(not(test))]
        {
            struct ComboWriter(std::fs::File, std::io::Stdout);

            impl std::io::Write for ComboWriter {
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    self.0.write(buf)?;
                    self.1.write(buf)
                }

                fn flush(&mut self) -> std::io::Result<()> {
                    self.0.flush()?;
                    self.1.flush()
                }
            }

            let file = std::fs::File::create("/tmp/xrizer.txt").unwrap();
            let writer = ComboWriter(file, std::io::stdout());
            builder.target(env_logger::Target::Pipe(Box::new(writer)));

            std::panic::set_hook(Box::new(|info| {
                log::error!("{info}");
                let backtrace = std::backtrace::Backtrace::force_capture();
                log::error!("Backtrace: \n{backtrace}");
                std::process::abort();
            }));
        }

        // safety: who cares lol
        unsafe {
            time::util::local_offset::set_soundness(time::util::local_offset::Soundness::Unsound)
        };

        builder
            .filter_level(log::LevelFilter::Info)
            .parse_default_env()
            .is_test(cfg!(test))
            .format(|buf, record| {
                use std::io::Write;
                use time::macros::format_description;

                let style = buf.default_level_style(record.level());
                let now = time::OffsetDateTime::now_local()
                    .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
                let now = now
                    .format(format_description!(
                        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]"
                    ))
                    .unwrap();

                write!(buf, "[{now} {style}{:5}{style:#}", record.level())?;
                if let Some(path) = record.module_path() {
                    write!(buf, " {}", path)?;
                }
                writeln!(buf, " {:?}] {}", std::thread::current().id(), record.args())
            })
            .init();
    });

    log::info!("Initializing XRizer");
}

#[no_mangle]
pub extern "C" fn VRClientCoreFactory(
    interface_name: *const c_char,
    return_code: *mut i32,
) -> *mut c_void {
    let interface = unsafe { CStr::from_ptr(interface_name) };
    ClientCore::new(interface)
        .map(|c| {
            if let Some(ret) = unsafe { return_code.as_mut() } {
                *ret = 0;
            }
            let vtable = match c.base.get().unwrap() {
                clientcore::Vtable::V2(v) => v as *const _ as *const vr::IVRClientCore002 as _,
                clientcore::Vtable::V3(v) => v as *const _ as *const vr::IVRClientCore003 as _,
            };
            // Leak it!
            let _ = Arc::into_raw(c);
            vtable
        })
        .unwrap_or(std::ptr::null_mut())
}

/// Needed for Proton, but seems unused.
#[no_mangle]
pub extern "C" fn HmdSystemFactory(
    _interface_name: *const c_char,
    _return_code: *mut i32,
) -> *mut c_void {
    unimplemented!()
}
