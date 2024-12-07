use openvr as vr;

#[derive(Default, macros::InterfaceImpl)]
#[interface = "IVRScreenshots"]
#[versions(001)]
pub struct Screenshots {
    vtables: Vtables,
}

impl vr::IVRScreenshots001_Interface for Screenshots {
    fn SubmitScreenshot(
        &self,
        _: vr::ScreenshotHandle_t,
        _: vr::EVRScreenshotType,
        _: *const std::os::raw::c_char,
        _: *const std::os::raw::c_char,
    ) -> vr::EVRScreenshotError {
        todo!()
    }
    fn TakeStereoScreenshot(
        &self,
        _: *mut vr::ScreenshotHandle_t,
        _: *const std::os::raw::c_char,
        _: *const std::os::raw::c_char,
    ) -> vr::EVRScreenshotError {
        todo!()
    }
    fn UpdateScreenshotProgress(
        &self,
        _: vr::ScreenshotHandle_t,
        _: f32,
    ) -> vr::EVRScreenshotError {
        todo!()
    }
    fn GetScreenshotPropertyFilename(
        &self,
        _: vr::ScreenshotHandle_t,
        _: vr::EVRScreenshotPropertyFilenames,
        _: *mut std::os::raw::c_char,
        _: u32,
        _: *mut vr::EVRScreenshotError,
    ) -> u32 {
        todo!()
    }
    fn GetScreenshotPropertyType(
        &self,
        _: vr::ScreenshotHandle_t,
        _: *mut vr::EVRScreenshotError,
    ) -> vr::EVRScreenshotType {
        todo!()
    }
    fn HookScreenshot(
        &self,
        _: *const vr::EVRScreenshotType,
        _: std::os::raw::c_int,
    ) -> vr::EVRScreenshotError {
        vr::EVRScreenshotError::None
    }
    fn RequestScreenshot(
        &self,
        _: *mut vr::ScreenshotHandle_t,
        _: vr::EVRScreenshotType,
        _: *const std::os::raw::c_char,
        _: *const std::os::raw::c_char,
    ) -> vr::EVRScreenshotError {
        todo!()
    }
}
