use crate::vr;

#[derive(Default, macros::InterfaceImpl)]
#[interface = "IVRChaperone"]
#[versions(004, 003)]
pub struct Chaperone {
    vtables: Vtables,
}

impl vr::IVRChaperone004_Interface for Chaperone {
    fn ResetZeroPose(&self, _: vr::ETrackingUniverseOrigin) {
        crate::warn_unimplemented!("ResetZeroPose");
    }
    fn ForceBoundsVisible(&self, _: bool) {
        todo!()
    }
    fn AreBoundsVisible(&self) -> bool {
        false
    }
    fn GetBoundsColor(
        &self,
        _: *mut vr::HmdColor_t,
        _: std::os::raw::c_int,
        _: f32,
        _: *mut vr::HmdColor_t,
    ) {
        todo!()
    }
    fn SetSceneColor(&self, _: vr::HmdColor_t) {
        todo!()
    }
    fn ReloadInfo(&self) {
        todo!()
    }
    fn GetPlayAreaRect(&self, rect: *mut vr::HmdQuad_t) -> bool {
        crate::warn_unimplemented!("GetPlayAreaRect");
        unsafe {
            *rect = Default::default();
        }
        false
    }
    fn GetPlayAreaSize(&self, size_x: *mut f32, size_z: *mut f32) -> bool {
        crate::warn_unimplemented!("GetPlayAreaSize");
        unsafe {
            *size_x = 1.0;
            *size_z = 1.0;
        };
        true
    }
    fn GetCalibrationState(&self) -> vr::ChaperoneCalibrationState {
        vr::ChaperoneCalibrationState::ChaperoneCalibrationState_OK
    }
}
