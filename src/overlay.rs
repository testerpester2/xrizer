use crate::vr;

#[derive(Default, macros::InterfaceImpl)]
#[interface = "IVROverlay"]
#[versions(027, 024, 021, 020, 019, 018, 016)]
pub struct OverlayMan {
    vtables: Vtables,
}

#[allow(unused_variables, non_snake_case)]
impl vr::IVROverlay027_Interface for OverlayMan {
    fn CloseMessageOverlay(&self) {
        todo!()
    }
    fn ShowMessageOverlay(
        &self,
        pchText: *const std::os::raw::c_char,
        pchCaption: *const std::os::raw::c_char,
        pchButton0Text: *const std::os::raw::c_char,
        pchButton1Text: *const std::os::raw::c_char,
        pchButton2Text: *const std::os::raw::c_char,
        pchButton3Text: *const std::os::raw::c_char,
    ) -> vr::VRMessageOverlayResponse {
        todo!()
    }
    fn SetKeyboardPositionForOverlay(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        avoidRect: vr::HmdRect2_t,
    ) {
        todo!()
    }
    fn SetKeyboardTransformAbsolute(
        &self,
        eTrackingOrigin: vr::ETrackingUniverseOrigin,
        pmatTrackingOriginToKeyboardTransform: *const vr::HmdMatrix34_t,
    ) {
        todo!()
    }
    fn HideKeyboard(&self) {
        todo!()
    }
    fn GetKeyboardText(&self, pchText: *mut std::os::raw::c_char, cchText: u32) -> u32 {
        todo!()
    }
    fn ShowKeyboardForOverlay(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        eInputMode: vr::EGamepadTextInputMode,
        eLineInputMode: vr::EGamepadTextInputLineMode,
        unFlags: u32,
        pchDescription: *const std::os::raw::c_char,
        unCharMax: u32,
        pchExistingText: *const std::os::raw::c_char,
        uUserValue: u64,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn ShowKeyboard(
        &self,
        eInputMode: vr::EGamepadTextInputMode,
        eLineInputMode: vr::EGamepadTextInputLineMode,
        unFlags: u32,
        pchDescription: *const std::os::raw::c_char,
        unCharMax: u32,
        pchExistingText: *const std::os::raw::c_char,
        uUserValue: u64,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetPrimaryDashboardDevice(&self) -> vr::TrackedDeviceIndex_t {
        todo!()
    }
    fn ShowDashboard(&self, pchOverlayToShow: *const std::os::raw::c_char) {
        todo!()
    }
    fn GetDashboardOverlaySceneProcess(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        punProcessId: *mut u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetDashboardOverlaySceneProcess(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        unProcessId: u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn IsActiveDashboardOverlay(&self, ulOverlayHandle: vr::VROverlayHandle_t) -> bool {
        todo!()
    }
    fn IsDashboardVisible(&self) -> bool {
        false
    }
    fn CreateDashboardOverlay(
        &self,
        pchOverlayKey: *const std::os::raw::c_char,
        pchOverlayFriendlyName: *const std::os::raw::c_char,
        pMainHandle: *mut vr::VROverlayHandle_t,
        pThumbnailHandle: *mut vr::VROverlayHandle_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayTextureSize(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pWidth: *mut u32,
        pHeight: *mut u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn ReleaseNativeOverlayHandle(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pNativeTextureHandle: *mut std::os::raw::c_void,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayTexture(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pNativeTextureHandle: *mut *mut std::os::raw::c_void,
        pNativeTextureRef: *mut std::os::raw::c_void,
        pWidth: *mut u32,
        pHeight: *mut u32,
        pNativeFormat: *mut u32,
        pAPIType: *mut vr::ETextureType,
        pColorSpace: *mut vr::EColorSpace,
        pTextureBounds: *mut vr::VRTextureBounds_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayFromFile(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pchFilePath: *const std::os::raw::c_char,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayRaw(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pvBuffer: *mut std::os::raw::c_void,
        unWidth: u32,
        unHeight: u32,
        unBytesPerPixel: u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn ClearOverlayTexture(&self, ulOverlayHandle: vr::VROverlayHandle_t) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayTexture(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pTexture: *const vr::Texture_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn ClearOverlayCursorPositionOverride(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayCursorPositionOverride(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pvCursor: *const vr::HmdVector2_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayCursor(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        ulCursorHandle: vr::VROverlayHandle_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn TriggerLaserMouseHapticVibration(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        fDurationSeconds: f32,
        fFrequency: f32,
        fAmplitude: f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayIntersectionMask(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pMaskPrimitives: *mut vr::VROverlayIntersectionMaskPrimitive_t,
        unNumMaskPrimitives: u32,
        unPrimitiveSize: u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn IsHoverTargetOverlay(&self, ulOverlayHandle: vr::VROverlayHandle_t) -> bool {
        todo!()
    }
    fn ComputeOverlayIntersection(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pParams: *const vr::VROverlayIntersectionParams_t,
        pResults: *mut vr::VROverlayIntersectionResults_t,
    ) -> bool {
        todo!()
    }
    fn SetOverlayMouseScale(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pvecMouseScale: *const vr::HmdVector2_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayMouseScale(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pvecMouseScale: *mut vr::HmdVector2_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayInputMethod(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        eInputMethod: vr::VROverlayInputMethod,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayInputMethod(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        peInputMethod: *mut vr::VROverlayInputMethod,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn PollNextOverlayEvent(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pEvent: *mut vr::VREvent_t,
        uncbVREvent: u32,
    ) -> bool {
        todo!()
    }
    fn WaitFrameSync(&self, nTimeoutMs: u32) -> vr::EVROverlayError {
        todo!()
    }
    fn GetTransformForOverlayCoordinates(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        eTrackingOrigin: vr::ETrackingUniverseOrigin,
        coordinatesInOverlay: vr::HmdVector2_t,
        pmatTransform: *mut vr::HmdMatrix34_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn IsOverlayVisible(&self, ulOverlayHandle: vr::VROverlayHandle_t) -> bool {
        todo!()
    }
    fn HideOverlay(&self, ulOverlayHandle: vr::VROverlayHandle_t) -> vr::EVROverlayError {
        todo!()
    }
    fn ShowOverlay(&self, ulOverlayHandle: vr::VROverlayHandle_t) -> vr::EVROverlayError {
        crate::warn_unimplemented!("ShowOverlay");
        vr::EVROverlayError::None
    }
    fn SetOverlayTransformProjection(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        eTrackingOrigin: vr::ETrackingUniverseOrigin,
        pmatTrackingOriginToOverlayTransform: *const vr::HmdMatrix34_t,
        pProjection: *const vr::VROverlayProjection_t,
        eEye: vr::EVREye,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayTransformCursor(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pvHotspot: *mut vr::HmdVector2_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayTransformCursor(
        &self,
        ulCursorOverlayHandle: vr::VROverlayHandle_t,
        pvHotspot: *const vr::HmdVector2_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayTransformTrackedDeviceComponent(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        punDeviceIndex: *mut vr::TrackedDeviceIndex_t,
        pchComponentName: *mut std::os::raw::c_char,
        unComponentNameSize: u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayTransformTrackedDeviceComponent(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        unDeviceIndex: vr::TrackedDeviceIndex_t,
        pchComponentName: *const std::os::raw::c_char,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayTransformTrackedDeviceRelative(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        punTrackedDevice: *mut vr::TrackedDeviceIndex_t,
        pmatTrackedDeviceToOverlayTransform: *mut vr::HmdMatrix34_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayTransformTrackedDeviceRelative(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        unTrackedDevice: vr::TrackedDeviceIndex_t,
        pmatTrackedDeviceToOverlayTransform: *const vr::HmdMatrix34_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayTransformAbsolute(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        peTrackingOrigin: *mut vr::ETrackingUniverseOrigin,
        pmatTrackingOriginToOverlayTransform: *mut vr::HmdMatrix34_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayTransformAbsolute(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        eTrackingOrigin: vr::ETrackingUniverseOrigin,
        pmatTrackingOriginToOverlayTransform: *const vr::HmdMatrix34_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayTransformType(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        peTransformType: *mut vr::VROverlayTransformType,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayTextureBounds(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pOverlayTextureBounds: *mut vr::VRTextureBounds_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayTextureBounds(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pOverlayTextureBounds: *const vr::VRTextureBounds_t,
    ) -> vr::EVROverlayError {
        crate::warn_unimplemented!("SetOverlayTextureBounds");
        vr::EVROverlayError::None
    }
    fn GetOverlayTextureColorSpace(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        peTextureColorSpace: *mut vr::EColorSpace,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayTextureColorSpace(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        eTextureColorSpace: vr::EColorSpace,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayPreCurvePitch(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pfRadians: *mut f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayPreCurvePitch(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        fRadians: f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayCurvature(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pfCurvature: *mut f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayCurvature(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        fCurvature: f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayWidthInMeters(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pfWidthInMeters: *mut f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayWidthInMeters(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        fWidthInMeters: f32,
    ) -> vr::EVROverlayError {
        crate::warn_unimplemented!("SetOverlayWidthInMeters");
        vr::EVROverlayError::None
    }
    fn GetOverlaySortOrder(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        punSortOrder: *mut u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlaySortOrder(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        unSortOrder: u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayTexelAspect(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pfTexelAspect: *mut f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayTexelAspect(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        fTexelAspect: f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayAlpha(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pfAlpha: *mut f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayAlpha(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        fAlpha: f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayColor(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pfRed: *mut f32,
        pfGreen: *mut f32,
        pfBlue: *mut f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayColor(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        fRed: f32,
        fGreen: f32,
        fBlue: f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayFlags(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pFlags: *mut u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayFlag(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        eOverlayFlag: vr::VROverlayFlags,
        pbEnabled: *mut bool,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayFlag(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        eOverlayFlag: vr::VROverlayFlags,
        bEnabled: bool,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayRenderingPid(&self, ulOverlayHandle: vr::VROverlayHandle_t) -> u32 {
        todo!()
    }
    fn SetOverlayRenderingPid(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        unPID: u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayErrorNameFromEnum(
        &self,
        error: vr::EVROverlayError,
    ) -> *const std::os::raw::c_char {
        todo!()
    }
    fn GetOverlayImageData(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pvBuffer: *mut std::os::raw::c_void,
        unBufferSize: u32,
        punWidth: *mut u32,
        punHeight: *mut u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayName(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pchName: *const std::os::raw::c_char,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayName(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pchValue: *mut std::os::raw::c_char,
        unBufferSize: u32,
        pError: *mut vr::EVROverlayError,
    ) -> u32 {
        todo!()
    }
    fn GetOverlayKey(
        &self,
        ulOverlayHandle: vr::VROverlayHandle_t,
        pchValue: *mut std::os::raw::c_char,
        unBufferSize: u32,
        pError: *mut vr::EVROverlayError,
    ) -> u32 {
        todo!()
    }
    fn DestroyOverlay(&self, ulOverlayHandle: vr::VROverlayHandle_t) -> vr::EVROverlayError {
        todo!()
    }
    fn CreateOverlay(
        &self,
        pchOverlayKey: *const std::os::raw::c_char,
        pchOverlayName: *const std::os::raw::c_char,
        pOverlayHandle: *mut vr::VROverlayHandle_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn FindOverlay(
        &self,
        pchOverlayKey: *const std::os::raw::c_char,
        pOverlayHandle: *mut vr::VROverlayHandle_t,
    ) -> vr::EVROverlayError {
        crate::warn_unimplemented!("FindOverlay");
        vr::EVROverlayError::None
    }
}

impl vr::IVROverlay024On027 for OverlayMan {
    fn SetOverlayTransformOverlayRelative(
        &self,
        _: vr::VROverlayHandle_t,
        _: vr::VROverlayHandle_t,
        _: *const vr::HmdMatrix34_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayTransformOverlayRelative(
        &self,
        _: vr::VROverlayHandle_t,
        _: *mut vr::VROverlayHandle_t,
        _: *mut vr::HmdMatrix34_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
}

impl vr::IVROverlay021On024 for OverlayMan {
    fn ShowKeyboardForOverlay(
        &self,
        _: vr::VROverlayHandle_t,
        _: vr::EGamepadTextInputMode,
        _: vr::EGamepadTextInputLineMode,
        _: *const std::os::raw::c_char,
        _: u32,
        _: *const std::os::raw::c_char,
        _: bool,
        _: u64,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn ShowKeyboard(
        &self,
        _: vr::EGamepadTextInputMode,
        _: vr::EGamepadTextInputLineMode,
        _: *const std::os::raw::c_char,
        _: u32,
        _: *const std::os::raw::c_char,
        _: bool,
        _: u64,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayRaw(
        &self,
        _: vr::VROverlayHandle_t,
        _: *mut std::os::raw::c_void,
        _: u32,
        _: u32,
        _: u32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayDualAnalogTransform(
        &self,
        _: vr::VROverlayHandle_t,
        _: vr::EDualAnalogWhich,
        _: *mut vr::HmdVector2_t,
        _: *mut f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayDualAnalogTransform(
        &self,
        _: vr::VROverlayHandle_t,
        _: vr::EDualAnalogWhich,
        _: *const vr::HmdVector2_t,
        _: f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayRenderModel(
        &self,
        _: vr::VROverlayHandle_t,
        _: *const std::os::raw::c_char,
        _: *const vr::HmdColor_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn GetOverlayRenderModel(
        &self,
        _: vr::VROverlayHandle_t,
        _: *mut std::os::raw::c_char,
        _: u32,
        _: *mut vr::HmdColor_t,
        _: *mut vr::EVROverlayError,
    ) -> u32 {
        todo!()
    }
}

impl vr::IVROverlay020On021 for OverlayMan {
    fn MoveGamepadFocusToNeighbor(
        &self,
        _: vr::EOverlayDirection,
        _: vr::VROverlayHandle_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayNeighbor(
        &self,
        _: vr::EOverlayDirection,
        _: vr::VROverlayHandle_t,
        _: vr::VROverlayHandle_t,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetGamepadFocusOverlay(&self, _: vr::VROverlayHandle_t) -> vr::EVROverlayError {
        todo!()
    }
    fn GetGamepadFocusOverlay(&self) -> vr::VROverlayHandle_t {
        todo!()
    }
    fn GetOverlayAutoCurveDistanceRangeInMeters(
        &self,
        _: vr::VROverlayHandle_t,
        _: *mut f32,
        _: *mut f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
    fn SetOverlayAutoCurveDistanceRangeInMeters(
        &self,
        _: vr::VROverlayHandle_t,
        _: f32,
        _: f32,
    ) -> vr::EVROverlayError {
        todo!()
    }
}

// The OpenVR commit messages mention that these functions just go through the standard overlay
// rendering path now.
impl vr::IVROverlay019On020 for OverlayMan {
    fn GetHighQualityOverlay(&self) -> vr::VROverlayHandle_t {
        unimplemented!()
    }
    fn SetHighQualityOverlay(&self, _: vr::VROverlayHandle_t) -> vr::EVROverlayError {
        unimplemented!()
    }
}

impl vr::IVROverlay018On019 for OverlayMan {
    #[inline]
    fn SetOverlayDualAnalogTransform(
        &self,
        overlay: vr::VROverlayHandle_t,
        which: vr::EDualAnalogWhich,
        center: *const vr::HmdVector2_t,
        radius: f32,
    ) -> vr::EVROverlayError {
        <Self as vr::IVROverlay021_Interface>::SetOverlayDualAnalogTransform(
            self, overlay, which, center, radius,
        )
    }
}

impl vr::IVROverlay016On018 for OverlayMan {
    fn HandleControllerOverlayInteractionAsMouse(
        &self,
        _: vr::VROverlayHandle_t,
        _: vr::TrackedDeviceIndex_t,
    ) -> bool {
        todo!()
    }
}
