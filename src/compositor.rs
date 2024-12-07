use crate::{
    clientcore::{Injected, Injector},
    input::Input,
    openxr_data::{self, OpenXrData, SessionData},
    overlay::OverlayMan,
    vulkan::VulkanData,
};
use ash::vk::{self, Handle};
use log::{debug, info, trace};
use openvr as vr;
use openxr as xr;
use std::sync::{Arc, Mutex};

struct FrameController {
    stream: xr::FrameStream<xr::vulkan::Vulkan>,
    waiter: xr::FrameWaiter,
    swapchain: xr::Swapchain<xr::vulkan::Vulkan>,
    image_index: usize,
    image_acquired: bool,
    should_render: bool,
    eyes_submitted: [Option<SubmittedEye>; 2],
    backend: GraphicsApi,
}

#[derive(Copy, Clone, Default)]
struct SubmittedEye {
    extent: xr::Extent2Di,
    flip_vertically: bool,
}

#[derive(Default)]
pub struct CompositorSessionData(Mutex<Option<FrameController>>);

#[derive(macros::InterfaceImpl)]
#[interface = "IVRCompositor"]
#[versions(028, 027, 026, 022, 020, 019)]
pub struct Compositor {
    vtables: Vtables,
    openxr: Arc<OpenXrData<Self>>,
    input: Injected<Input<Self>>,
    tmp_backend: Mutex<Option<GraphicsApi>>,
    swapchain_create_info: Mutex<xr::SwapchainCreateInfo<xr::vulkan::Vulkan>>,
    overlays: Injected<OverlayMan>,
}

impl Compositor {
    pub fn new(openxr: Arc<OpenXrData<Self>>, injector: &Injector) -> Self {
        Self {
            vtables: Default::default(),
            openxr,
            input: injector.inject(),
            tmp_backend: Mutex::default(),
            overlays: injector.inject(),
            swapchain_create_info: Mutex::new(
                // This will be overwritten before actually creating our swapchain.
                xr::SwapchainCreateInfo {
                    create_flags: Default::default(),
                    usage_flags: Default::default(),
                    format: 0,
                    sample_count: 0,
                    width: 0,
                    height: 0,
                    face_count: 0,
                    array_size: 0,
                    mip_count: 0,
                },
            ),
        }
    }

    /// Starts a frame if we've created our frame controller.
    fn maybe_start_frame(&self, session_data: &SessionData) {
        let mut frame_lock = session_data.comp_data.0.lock().unwrap();
        let Some(ctrl) = frame_lock.as_mut() else {
            debug!("no frame controller - not starting frame");
            return;
        };

        if ctrl.image_acquired {
            ctrl.swapchain.release_image().unwrap();
        }

        ctrl.image_index = ctrl
            .swapchain
            .acquire_image()
            .expect("Failed to acquire swapchain image") as usize;

        trace!("waiting image");
        ctrl.swapchain
            .wait_image(xr::Duration::INFINITE)
            .expect("Failed to wait for swapchain image");

        ctrl.image_acquired = true;
        let frame_state = ctrl.waiter.wait().unwrap();
        ctrl.should_render = frame_state.should_render;
        self.openxr
            .display_time
            .set(frame_state.predicted_display_time);
        ctrl.stream.begin().unwrap();
        ctrl.eyes_submitted = [None; 2];
        trace!("frame begin");
    }

    fn initialize_real_session(&self, texture: &vr::Texture_t, bounds: vr::VRTextureBounds_t) {
        let backend = GraphicsApi::new(texture);
        *self.swapchain_create_info.lock().unwrap() =
            backend.get_swapchain_create_info(texture, bounds);

        *self.tmp_backend.lock().unwrap() = Some(backend);

        self.openxr.restart_session(); // calls init_frame_controller
    }
}

impl openxr_data::Compositor for Compositor {
    fn pre_session_restart(
        &self,
        data: CompositorSessionData,
    ) -> openxr::vulkan::SessionCreateInfo {
        self.tmp_backend
            .lock()
            .unwrap()
            .get_or_insert_with(|| {
                data.0
                    .lock()
                    .unwrap()
                    .take()
                    .expect("one of tmp backend or frame controller should be setup")
                    .backend
            })
            .as_session_create_info()
    }

    fn init_frame_controller(
        &self,
        session_data: &SessionData,
        waiter: xr::FrameWaiter,
        stream: xr::FrameStream<xr::vulkan::Vulkan>,
    ) {
        // This function is called while a write lock is called on the session, and as such should
        // not use self.openxr.session_data.get().

        let mut backend = self
            .tmp_backend
            .lock()
            .unwrap()
            .take()
            .expect("tmp backend should be set before init_frame_controller is called");

        let swapchain = session_data
            .session
            .create_swapchain(&self.swapchain_create_info.lock().unwrap())
            .expect("Failed to create swapchain");

        let images: Vec<vk::Image> = swapchain
            .enumerate_images()
            .expect("Failed to enumerate swapchain images")
            .into_iter()
            .map(vk::Image::from_raw)
            .collect();

        backend.post_swapchain_create(images);

        *session_data.comp_data.0.lock().unwrap() = Some(FrameController {
            stream,
            waiter,
            swapchain,
            image_index: 0,
            image_acquired: false,
            should_render: false,
            eyes_submitted: Default::default(),
            backend,
        });

        self.maybe_start_frame(session_data);
    }
}

#[allow(non_snake_case)]
impl vr::IVRCompositor028_Interface for Compositor {
    fn GetPosesForFrame(
        &self,
        _unPosePredictionID: u32,
        _pPoseArray: *mut vr::TrackedDevicePose_t,
        _unPoseArrayCount: u32,
    ) -> vr::EVRCompositorError {
        todo!()
    }
    fn GetLastPosePredictionIDs(
        &self,
        _pRenderPosePredictionID: *mut u32,
        _pGamePosePredictionID: *mut u32,
    ) -> vr::EVRCompositorError {
        crate::warn_unimplemented!("GetLastPosePredictionIDs");
        vr::EVRCompositorError::None
    }
    fn GetCompositorBenchmarkResults(
        &self,
        _pBenchmarkResults: *mut vr::Compositor_BenchmarkResults,
        _nSizeOfBenchmarkResults: u32,
    ) -> bool {
        crate::warn_unimplemented!("GetCompositorBenchmarkResults");
        false
    }
    fn ClearStageOverride(&self) {}
    fn SetStageOverride_Async(
        &self,
        _pchRenderModelPath: *const std::ffi::c_char,
        _pTransform: *const vr::HmdMatrix34_t,
        _pRenderSettings: *const vr::Compositor_StageRenderSettings,
        _nSizeOfRenderSettings: u32,
    ) -> vr::EVRCompositorError {
        crate::warn_unimplemented!("SetStageOverride_Async");
        vr::EVRCompositorError::None
    }
    fn IsCurrentSceneFocusAppLoading(&self) -> bool {
        false
    }
    fn IsMotionSmoothingSupported(&self) -> bool {
        todo!()
    }
    fn IsMotionSmoothingEnabled(&self) -> bool {
        todo!()
    }
    fn SubmitExplicitTimingData(&self) -> vr::EVRCompositorError {
        crate::warn_unimplemented!("SubmitExplicitTimingData");
        vr::EVRCompositorError::None
    }
    fn SetExplicitTimingMode(&self, _eTimingMode: vr::EVRCompositorTimingMode) {
        crate::warn_unimplemented!("SetExplicitTimingMode");
    }

    fn GetVulkanDeviceExtensionsRequired(
        &self,
        _physical_device: *mut vr::VkPhysicalDevice_T,
        buffer: *mut std::ffi::c_char,
        buffer_size: u32,
    ) -> u32 {
        let exts = self
            .openxr
            .instance
            .vulkan_legacy_device_extensions(self.openxr.system_id)
            .unwrap();
        log::debug!("required device extensions: {exts}");
        let bytes = unsafe { &*(exts.as_bytes() as *const [u8] as *const [std::ffi::c_char]) };
        if !buffer.is_null() && buffer_size > 0 {
            let size = buffer_size as usize;
            let ret_buf = unsafe { std::slice::from_raw_parts_mut(buffer, size) };
            ret_buf[0..size - 1].copy_from_slice(&bytes[0..size - 1]);
            ret_buf[size - 1] = 0;
        }
        exts.len() as u32 + 1
    }

    fn GetVulkanInstanceExtensionsRequired(
        &self,
        buffer: *mut std::ffi::c_char,
        buffer_size: u32,
    ) -> u32 {
        let exts = self
            .openxr
            .instance
            .vulkan_legacy_instance_extensions(self.openxr.system_id)
            .unwrap();
        log::debug!("required instance extensions: {exts}");
        let bytes = unsafe { &*(exts.as_bytes() as *const [u8] as *const [std::ffi::c_char]) };
        if !buffer.is_null() && buffer_size > 0 {
            let size = buffer_size as usize;
            let ret_buf = unsafe { std::slice::from_raw_parts_mut(buffer, size) };
            ret_buf[0..size - 1].copy_from_slice(&bytes[0..size - 1]);
            ret_buf[size - 1] = 0;
        }
        exts.len() as u32 + 1
    }

    fn UnlockGLSharedTextureForAccess(&self, _glSharedTextureHandle: vr::glSharedTextureHandle_t) {
        todo!()
    }
    fn LockGLSharedTextureForAccess(&self, _glSharedTextureHandle: vr::glSharedTextureHandle_t) {
        todo!()
    }
    fn ReleaseSharedGLTexture(
        &self,
        _glTextureId: vr::glUInt_t,
        _glSharedTextureHandle: vr::glSharedTextureHandle_t,
    ) -> bool {
        todo!()
    }
    fn GetMirrorTextureGL(
        &self,
        _eEye: vr::EVREye,
        _pglTextureId: *mut vr::glUInt_t,
        _pglSharedTextureHandle: *mut vr::glSharedTextureHandle_t,
    ) -> vr::EVRCompositorError {
        todo!()
    }
    fn ReleaseMirrorTextureD3D11(&self, _pD3D11ShaderResourceView: *mut std::ffi::c_void) {
        todo!()
    }
    fn GetMirrorTextureD3D11(
        &self,
        _eEye: vr::EVREye,
        _pD3D11DeviceOrResource: *mut std::ffi::c_void,
        _ppD3D11ShaderResourceView: *mut *mut std::ffi::c_void,
    ) -> vr::EVRCompositorError {
        todo!()
    }
    fn SuspendRendering(&self, _bSuspend: bool) {
        crate::warn_unimplemented!("SuspendRendering");
    }
    fn ForceReconnectProcess(&self) {
        todo!()
    }
    fn ForceInterleavedReprojectionOn(&self, _: bool) {
        crate::warn_unimplemented!("ForceInterleavedReprojectionOn");
    }
    fn ShouldAppRenderWithLowResources(&self) -> bool {
        // TODO
        false
    }
    fn CompositorDumpImages(&self) {
        todo!()
    }
    fn IsMirrorWindowVisible(&self) -> bool {
        todo!()
    }
    fn HideMirrorWindow(&self) {
        todo!()
    }
    fn ShowMirrorWindow(&self) {
        todo!()
    }
    fn CanRenderScene(&self) -> bool {
        true
    }
    fn GetLastFrameRenderer(&self) -> u32 {
        todo!()
    }
    fn GetCurrentSceneFocusProcess(&self) -> u32 {
        todo!()
    }
    fn IsFullscreen(&self) -> bool {
        todo!()
    }
    fn CompositorQuit(&self) {
        todo!()
    }
    fn CompositorGoToBack(&self) {
        todo!()
    }
    fn CompositorBringToFront(&self) {
        todo!()
    }
    fn ClearSkyboxOverride(&self) {
        crate::warn_unimplemented!("ClearSkyboxOverride");
    }
    fn SetSkyboxOverride(
        &self,
        _pTextures: *const vr::Texture_t,
        _unTextureCount: u32,
    ) -> vr::EVRCompositorError {
        crate::warn_unimplemented!("SetSkyboxOverride");
        vr::EVRCompositorError::None
    }
    fn GetCurrentGridAlpha(&self) -> f32 {
        0.0
    }
    fn FadeGrid(&self, _fSeconds: f32, _bFadeGridIn: bool) {
        crate::warn_unimplemented!("FadeGrid");
    }
    fn GetCurrentFadeColor(&self, _bBackground: bool) -> vr::HmdColor_t {
        todo!()
    }
    fn FadeToColor(
        &self,
        _fSeconds: f32,
        _fRed: f32,
        _fGreen: f32,
        _fBlue: f32,
        _fAlpha: f32,
        _bBackground: bool,
    ) {
        crate::warn_unimplemented!("FadeToColor");
    }
    fn GetCumulativeStats(
        &self,
        _pStats: *mut vr::Compositor_CumulativeStats,
        _nStatsSizeInBytes: u32,
    ) {
        todo!()
    }
    fn GetFrameTimeRemaining(&self) -> f32 {
        todo!()
    }
    fn GetFrameTimings(&self, _pTiming: *mut vr::Compositor_FrameTiming, _nFrames: u32) -> u32 {
        todo!()
    }
    fn GetFrameTiming(&self, _pTiming: *mut vr::Compositor_FrameTiming, _unFramesAgo: u32) -> bool {
        crate::warn_unimplemented!("GetFrameTiming");
        false
    }
    fn PostPresentHandoff(&self) {
        crate::warn_unimplemented!("PostPresentHandoff");
    }
    fn ClearLastSubmittedFrame(&self) {
        crate::warn_unimplemented!("ClearLastSubmittedFrame");
    }
    fn SubmitWithArrayIndex(
        &self,
        _eEye: vr::EVREye,
        _pTexture: *const vr::Texture_t,
        _unTextureArrayIndex: u32,
        _pBounds: *const vr::VRTextureBounds_t,
        _nSubmitFlags: vr::EVRSubmitFlags,
    ) -> vr::EVRCompositorError {
        todo!()
    }

    fn Submit(
        &self,
        eye: vr::EVREye,
        texture: *const vr::Texture_t,
        bounds: *const vr::VRTextureBounds_t,
        submit_flags: vr::EVRSubmitFlags,
    ) -> vr::EVRCompositorError {
        let bounds = unsafe { bounds.as_ref() }
            .copied()
            .unwrap_or(vr::VRTextureBounds_t {
                uMin: 0.0,
                vMin: 0.0,
                uMax: 1.0,
                vMax: 1.0,
            });

        // Superhot passes crazy bounds on startup.
        if !bounds.valid() {
            return vr::EVRCompositorError::InvalidBounds;
        }

        let Some(texture) = (unsafe { texture.as_ref() }) else {
            return vr::EVRCompositorError::InvalidTexture;
        };

        let mut session_lock = self.openxr.session_data.get();
        let mut frame_lock = session_lock.comp_data.0.lock().unwrap();

        let mut ctrl = match frame_lock.as_mut() {
            Some(ctrl) => ctrl,
            None => {
                drop(frame_lock);
                drop(session_lock);

                info!("Received game texture, restarting session with new data");
                self.initialize_real_session(texture, bounds);

                session_lock = self.openxr.session_data.get();
                frame_lock = session_lock.comp_data.0.lock().unwrap();
                frame_lock.as_mut().unwrap()
            }
        };

        // No Man's Sky does this.
        if ctrl.eyes_submitted[eye as usize].is_some() {
            return vr::EVRCompositorError::AlreadySubmitted;
        }

        ctrl.eyes_submitted[eye as usize] = if ctrl.should_render {
            // Make sure our image dimensions haven't changed.
            let mut last_info = self.swapchain_create_info.lock().unwrap();
            if !ctrl
                .backend
                .is_usable_swapchain(&last_info, texture, bounds)
            {
                info!("recreating swapchain (for {eye:?})");
                *last_info = ctrl.backend.get_swapchain_create_info(texture, bounds);
                let FrameController {
                    stream,
                    waiter,
                    backend,
                    ..
                } = frame_lock.take().unwrap();
                drop(frame_lock);
                drop(session_lock);
                drop(last_info);
                *self.tmp_backend.lock().unwrap() = Some(backend);
                // init_frame_controller eventually calls xrBeginFrame again without calling
                // xrEndFrame, but this is legal.
                <Self as openxr_data::Compositor>::init_frame_controller(
                    self,
                    &self.openxr.session_data.get(),
                    waiter,
                    stream,
                );
                session_lock = self.openxr.session_data.get();
                frame_lock = session_lock.comp_data.0.lock().unwrap();
                ctrl = frame_lock.as_mut().unwrap()
            }
            Some(SubmittedEye {
                extent: ctrl.backend.copy_texture_to_swapchain(
                    eye,
                    texture,
                    bounds,
                    ctrl.image_index,
                    submit_flags,
                ),
                flip_vertically: bounds.vertically_flipped(),
            })
        } else {
            Some(Default::default())
        };

        trace!("submitted {eye:?}");
        if !ctrl.eyes_submitted.iter().all(|eye| eye.is_some()) {
            return vr::EVRCompositorError::None;
        }
        // Both eyes submitted: show our images

        trace!("releasing image");
        ctrl.swapchain.release_image().unwrap();
        ctrl.image_acquired = false;

        let mut proj_layer_views = Vec::new();
        if ctrl.should_render {
            let (flags, views) = session_lock
                .session
                .locate_views(
                    xr::ViewConfigurationType::PRIMARY_STEREO,
                    self.openxr.display_time.get(),
                    session_lock.tracking_space(),
                )
                .expect("Couldn't locate views");

            proj_layer_views = views
                .into_iter()
                .enumerate()
                .map(|(eye_index, view)| {
                    let pose = xr::Posef {
                        orientation: if flags.contains(xr::ViewStateFlags::ORIENTATION_VALID) {
                            view.pose.orientation
                        } else {
                            xr::Quaternionf::IDENTITY
                        },
                        position: if flags.contains(xr::ViewStateFlags::POSITION_VALID) {
                            view.pose.position
                        } else {
                            xr::Vector3f::default()
                        },
                    };

                    let SubmittedEye {
                        extent,
                        flip_vertically,
                    } = ctrl.eyes_submitted[eye_index].unwrap();
                    let mut fov = view.fov;
                    if flip_vertically {
                        std::mem::swap(&mut fov.angle_up, &mut fov.angle_down);
                    }

                    let sub_image = xr::SwapchainSubImage::new()
                        .swapchain(&ctrl.swapchain)
                        .image_array_index(eye_index as u32)
                        .image_rect(xr::Rect2Di {
                            extent,
                            offset: xr::Offset2Di::default(),
                        });

                    xr::CompositionLayerProjectionView::new()
                        .fov(fov)
                        .pose(pose)
                        .sub_image(sub_image)
                })
                .collect()
        }

        let mut proj_layer = None;
        if !proj_layer_views.is_empty() {
            proj_layer = Some(
                xr::CompositionLayerProjection::new()
                    .space(session_lock.tracking_space())
                    .views(&proj_layer_views),
            );
        }

        let mut layers: Vec<&xr::CompositionLayerBase<_>> = Vec::new();
        if let Some(l) = proj_layer.as_ref() {
            layers.push(l);
        }
        let overlays;
        if let Some(overlay_man) = self.overlays.get() {
            overlays = overlay_man.get_layers(&session_lock);
            layers.extend(overlays.iter().map(std::ops::Deref::deref));
        }

        ctrl.stream
            .end(
                self.openxr.display_time.get(),
                xr::EnvironmentBlendMode::OPAQUE,
                &layers,
            )
            .unwrap();
        trace!("frame submitted");

        vr::EVRCompositorError::None
    }

    fn GetLastPoseForTrackedDeviceIndex(
        &self,
        _unDeviceIndex: vr::TrackedDeviceIndex_t,
        _pOutputPose: *mut vr::TrackedDevicePose_t,
        _pOutputGamePose: *mut vr::TrackedDevicePose_t,
    ) -> vr::EVRCompositorError {
        todo!()
    }
    fn GetLastPoses(
        &self,
        render_pose_array: *mut vr::TrackedDevicePose_t,
        render_pose_count: u32,
        game_pose_array: *mut vr::TrackedDevicePose_t,
        game_pose_count: u32,
    ) -> vr::EVRCompositorError {
        if render_pose_count == 0 {
            return vr::EVRCompositorError::None;
        }
        let render_poses = unsafe {
            std::slice::from_raw_parts_mut(render_pose_array, render_pose_count as usize)
        };
        self.input
            .force(|_| Input::new(self.openxr.clone()))
            .get_poses(render_poses, None);

        // Not entirely sure how the game poses are supposed to differ from the render poses,
        // but a lot of games use the game pose array for controller positions.
        if game_pose_count > 0 {
            let game_poses = unsafe {
                std::slice::from_raw_parts_mut(game_pose_array, game_pose_count as usize)
            };
            assert!(game_poses.len() <= render_poses.len());
            game_poses.copy_from_slice(&render_poses[0..game_poses.len()]);
        }

        vr::EVRCompositorError::None
    }

    fn WaitGetPoses(
        &self,
        render_pose_array: *mut vr::TrackedDevicePose_t,
        render_pose_count: u32,
        game_pose_array: *mut vr::TrackedDevicePose_t,
        game_pose_count: u32,
    ) -> vr::EVRCompositorError {
        // This should be called every frame - we must regularly poll events
        self.openxr.poll_events();
        {
            let session_data = self.openxr.session_data.get();
            self.maybe_start_frame(&session_data);
        }
        if let Some(input) = self.input.get() {
            input.frame_start_update();
        }

        self.GetLastPoses(
            render_pose_array,
            render_pose_count,
            game_pose_array,
            game_pose_count,
        )
    }

    fn GetTrackingSpace(&self) -> vr::ETrackingUniverseOrigin {
        self.openxr.get_tracking_space()
    }

    fn SetTrackingSpace(&self, origin: vr::ETrackingUniverseOrigin) {
        self.openxr.set_tracking_space(origin);
    }
}

impl vr::IVRCompositor026On027 for Compositor {
    fn FadeGrid(&self, seconds: f32, fade_in: bool) {
        <Self as vr::IVRCompositor028_Interface>::FadeGrid(self, seconds, fade_in);
    }
}

enum GraphicsApi {
    Vulkan(VulkanData),
    #[cfg(test)]
    Fake(tests::FakeGraphicsData),
}

impl GraphicsApi {
    fn new(texture: &vr::Texture_t) -> Self {
        match texture.eType {
            vr::ETextureType::Vulkan => {
                let vk_texture = unsafe { &*(texture.handle as *const vr::VRVulkanTextureData_t) };
                Self::Vulkan(VulkanData::new(vk_texture))
            }
            #[cfg(test)]
            vr::ETextureType::Reserved => Self::Fake(tests::FakeGraphicsData::new(texture)),
            other => panic!("Unsupported texture type: {other:?}"),
        }
    }

    fn as_session_create_info(&self) -> xr::vulkan::SessionCreateInfo {
        match self {
            Self::Vulkan(vk) => vk.as_session_create_info(),
            #[cfg(test)]
            Self::Fake(f) => f.as_session_create_info(),
        }
    }

    fn get_swapchain_create_info(
        &self,
        texture: &vr::Texture_t,
        bounds: vr::VRTextureBounds_t,
    ) -> xr::SwapchainCreateInfo<xr::vulkan::Vulkan> {
        match self {
            Self::Vulkan(_) => {
                assert_eq!(texture.eType, vr::ETextureType::Vulkan);
                let vk_texture = unsafe { &*(texture.handle as *const vr::VRVulkanTextureData_t) };
                VulkanData::get_swapchain_create_info(vk_texture, bounds, texture.eColorSpace)
            }
            #[cfg(test)]
            Self::Fake(f) => f.check_swapchain(texture, bounds),
        }
    }

    fn is_usable_swapchain(
        &self,
        create_info: &xr::SwapchainCreateInfo<xr::vulkan::Vulkan>,
        texture: &vr::Texture_t,
        bounds: vr::VRTextureBounds_t,
    ) -> bool {
        let new_info = self.get_swapchain_create_info(texture, bounds);

        create_info.format == new_info.format
            && create_info.width == new_info.width
            && create_info.height == new_info.height
            && create_info.array_size == new_info.array_size
            && create_info.sample_count == new_info.sample_count
    }

    fn post_swapchain_create(&mut self, images: Vec<vk::Image>) {
        match self {
            Self::Vulkan(vk) => vk.post_swapchain_create(images),
            #[cfg(test)]
            Self::Fake(_) => {}
        }
    }

    fn copy_texture_to_swapchain(
        &mut self,
        eye: vr::EVREye,
        texture: &vr::Texture_t,
        bounds: vr::VRTextureBounds_t,
        image_index: usize,
        submit_flags: vr::EVRSubmitFlags,
    ) -> xr::Extent2Di {
        match self {
            Self::Vulkan(vk) => {
                assert_eq!(texture.eType, vr::ETextureType::Vulkan);
                vk.copy_texture_to_swapchain(
                    eye,
                    texture.handle.cast::<vr::VRVulkanTextureData_t>(),
                    bounds,
                    image_index,
                    submit_flags,
                )
            }
            #[cfg(test)]
            Self::Fake(_) => xr::Extent2Di::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::thread_local;
    use vr::EVRCompositorError::*;
    use vr::IVRCompositor028_Interface;

    pub struct FakeGraphicsData(Arc<VulkanData>);
    thread_local! {
        static WIDTH: Cell<u32> = const { Cell::new(10) };
    }

    impl FakeGraphicsData {
        fn texture(data: &Arc<VulkanData>) -> vr::Texture_t {
            vr::Texture_t {
                eType: vr::ETextureType::Reserved,
                handle: Arc::into_raw(data.clone()) as _,
                eColorSpace: vr::EColorSpace::Auto,
            }
        }

        pub fn new(texture: &vr::Texture_t) -> Self {
            assert_eq!(texture.eType, vr::ETextureType::Reserved);
            let ptr = texture.handle as *const VulkanData;
            let vk = unsafe {
                Arc::increment_strong_count(ptr);
                Arc::from_raw(ptr)
            };
            Self(vk)
        }

        pub fn check_swapchain(
            &self,
            texture: &vr::Texture_t,
            _bounds: vr::VRTextureBounds_t,
        ) -> xr::SwapchainCreateInfo<xr::vulkan::Vulkan> {
            assert_eq!(texture.eType, vr::ETextureType::Reserved);
            xr::SwapchainCreateInfo {
                create_flags: xr::SwapchainCreateFlags::EMPTY,
                usage_flags: xr::SwapchainUsageFlags::EMPTY,
                format: 0,
                sample_count: 1,
                width: WIDTH.get(),
                height: 10,
                face_count: 1,
                array_size: 2,
                mip_count: 1,
            }
        }

        pub fn as_session_create_info(&self) -> xr::vulkan::SessionCreateInfo {
            self.0.as_session_create_info()
        }
    }

    struct Fixture {
        comp: Arc<Compositor>,
        vk: Arc<VulkanData>,
    }

    impl Fixture {
        fn new() -> Self {
            let xr = Arc::new(OpenXrData::new(&Injector::default()).unwrap());
            let vk = Arc::new(VulkanData::new_temporary(&xr.instance, xr.system_id));
            let comp = Arc::new(Compositor::new(xr.clone(), &Injector::default()));
            xr.compositor.set(Arc::downgrade(&comp));

            Self { comp, vk }
        }

        pub fn wait_get_poses(&self) -> vr::EVRCompositorError {
            self.comp
                .WaitGetPoses(std::ptr::null_mut(), 0, std::ptr::null_mut(), 0)
        }

        pub fn submit(&self, eye: vr::EVREye) -> vr::EVRCompositorError {
            self.comp.Submit(
                eye,
                &FakeGraphicsData::texture(&self.vk),
                std::ptr::null(),
                vr::EVRSubmitFlags::Default,
            )
        }
    }

    #[test]
    fn bad_bounds() {
        let f = Fixture::new();

        let bad_bounds = [
            // bound greater than 1 (Superhot does this)
            vr::VRTextureBounds_t {
                vMin: 0.0,
                vMax: 1.1,
                uMin: 0.0,
                uMax: 1.0,
            },
            // bound greater than 1
            vr::VRTextureBounds_t {
                vMin: 0.0,
                vMax: 1.0,
                uMin: 0.0,
                uMax: 1.1,
            },
            // negative bound
            vr::VRTextureBounds_t {
                vMin: -0.1,
                vMax: 1.0,
                uMin: 0.0,
                uMax: 1.0,
            },
            // min/max equal
            vr::VRTextureBounds_t {
                vMin: 1.0,
                vMax: 1.0,
                uMin: 0.0,
                uMax: 1.0,
            },
        ];

        for bound in bad_bounds {
            assert_eq!(
                f.comp.Submit(
                    vr::EVREye::Left,
                    &FakeGraphicsData::texture(&f.vk),
                    &bound,
                    vr::EVRSubmitFlags::Default,
                ),
                InvalidBounds,
                "Bound didn't return InvalidBounds: {bound:?}"
            );
        }
    }

    #[test]
    fn allow_flipped_bounds() {
        let Fixture { comp, .. } = Fixture::new();

        assert_ne!(
            comp.Submit(
                vr::EVREye::Left,
                std::ptr::null(),
                &vr::VRTextureBounds_t {
                    uMin: 0.0,
                    vMin: 1.0,
                    uMax: 1.0,
                    vMax: 0.0
                },
                vr::EVRSubmitFlags::Default
            ),
            InvalidBounds
        );
    }

    #[test]
    fn error_on_submit_without_waitgetposes() {
        let f = Fixture::new();

        assert_eq!(f.wait_get_poses(), None);
        assert_eq!(f.submit(vr::EVREye::Left), None);
        assert_eq!(f.submit(vr::EVREye::Right), None);
        assert_eq!(f.submit(vr::EVREye::Left), AlreadySubmitted);

        assert_eq!(f.wait_get_poses(), None);
        assert_eq!(f.submit(vr::EVREye::Left), None);
    }

    #[test]
    fn allow_waitgetposes_without_submit() {
        let f = Fixture::new();

        // Enable frame controller
        assert_eq!(f.wait_get_poses(), None);
        assert_eq!(f.submit(vr::EVREye::Left), None);

        assert_eq!(f.wait_get_poses(), None);
        assert_eq!(f.wait_get_poses(), None);
    }

    #[test]
    fn recreate_swapchain() {
        let f = Fixture::new();
        fakexr::should_render_next_frame(f.comp.openxr.instance.as_raw(), true);

        assert_eq!(f.wait_get_poses(), None);
        assert_eq!(f.submit(vr::EVREye::Left), None);
        WIDTH.set(40);
        assert_eq!(f.submit(vr::EVREye::Right), None);
    }
}
