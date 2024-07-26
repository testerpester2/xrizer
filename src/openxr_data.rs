use crate::{
    clientcore::{Injected, Injector},
    vr,
    vulkan::VulkanData,
};
use log::info;
use openxr as xr;
use std::mem::ManuallyDrop;
use std::sync::{
    atomic::{AtomicBool, AtomicI64, Ordering},
    RwLock,
};

pub trait Compositor: crate::InterfaceImpl {
    fn init_frame_controller(
        &self,
        session: &SessionData,
        waiter: xr::FrameWaiter,
        stream: xr::FrameStream<xr::vulkan::Vulkan>,
    );

    fn current_session_create_info(&self) -> xr::vulkan::SessionCreateInfo;
}

pub type RealOpenXrData = OpenXrData<crate::compositor::Compositor>;
pub struct OpenXrData<C: Compositor> {
    _entry: xr::Entry,
    pub instance: xr::Instance,
    pub system_id: xr::SystemId,
    pub session_data: SessionReadGuard,
    pub display_time: AtomicXrTime,
    pub left_hand: HandInfo,
    pub right_hand: HandInfo,

    /// should only be externally accessed for testing
    pub(crate) input: Injected<crate::input::Input<C>>,
    pub(crate) compositor: Injected<C>,
}

impl<C: Compositor> Drop for OpenXrData<C> {
    fn drop(&mut self) {
        self.end_session();
        unsafe {
            ManuallyDrop::drop(&mut *self.session_data.0.get_mut().unwrap());
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)] // Results aren't used, but they're printed
pub enum InitError {
    InstanceCreationFailed(xr::sys::Result),
    SystemCreationFailed(xr::sys::Result),
    SessionCreationFailed(SessionCreationError),
}

impl From<SessionCreationError> for InitError {
    fn from(value: SessionCreationError) -> Self {
        Self::SessionCreationFailed(value)
    }
}

impl<C: Compositor> OpenXrData<C> {
    pub fn new(injector: &Injector) -> Result<Self, InitError> {
        #[cfg(not(test))]
        let entry = xr::Entry::linked();

        #[cfg(test)]
        let entry =
            unsafe { xr::Entry::from_get_instance_proc_addr(fakexr::get_instance_proc_addr) }
                .unwrap();
        let mut exts = xr::ExtensionSet::default();
        exts.khr_vulkan_enable = true;
        let instance = entry
            .create_instance(
                &xr::ApplicationInfo {
                    application_name: "XRizer",
                    application_version: 0,
                    ..Default::default()
                },
                &exts,
                &[],
            )
            .map_err(|e| InitError::InstanceCreationFailed(e))?;

        let system_id = instance
            .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
            .map_err(|e| InitError::SystemCreationFailed(e))?;

        let session_data = SessionReadGuard(RwLock::new(ManuallyDrop::new(
            SessionData::new(
                &instance,
                system_id,
                vr::ETrackingUniverseOrigin::TrackingUniverseSeated,
                None,
            )?
            .0,
        )));

        let left_hand = HandInfo::new(&instance, "/user/hand/left");
        let right_hand = HandInfo::new(&instance, "/user/hand/right");

        Ok(Self {
            _entry: entry,
            instance,
            system_id,
            session_data,
            display_time: AtomicXrTime(1.into()),
            left_hand,
            right_hand,
            input: injector.inject(),
            compositor: injector.inject(),
        })
    }

    pub fn poll_events(&self) {
        let mut buf = xr::EventDataBuffer::new();
        while let Some(event) = self.instance.poll_event(&mut buf).unwrap() {
            match event {
                xr::Event::SessionStateChanged(event) => {
                    self.session_data.0.write().unwrap().state = event.state();
                    info!("OpenXR session state changed: {:?}", event.state());
                }
                xr::Event::InteractionProfileChanged(_) => {
                    let session = self.session_data.get();
                    for info in [&self.left_hand, &self.right_hand] {
                        let profile = session
                            .session
                            .current_interaction_profile(info.path)
                            .unwrap();

                        let profile = match profile {
                            xr::Path::NULL => {
                                info.connected.store(false, Ordering::Relaxed);
                                "<null>".to_owned()
                            }
                            path => {
                                info.connected.store(true, Ordering::Relaxed);
                                self.instance.path_to_string(path).unwrap()
                            }
                        };

                        info!("{} interaction profile changed: {}", info.s, profile);
                    }
                }
                _ => {
                    info!("unknown event");
                }
            }
        }
    }

    /// TODO: support non vulkan session
    pub fn restart_session(&self) {
        self.end_session();
        let mut session_guard = self.session_data.0.write().unwrap();

        let origin = session_guard.current_origin;
        // We need to destroy the old session before creating the new one.
        let _ = unsafe { ManuallyDrop::take(&mut *session_guard) };

        let comp = self
            .compositor
            .get()
            .expect("Session is being restarted, but compositor has not been set up!");

        let info = comp.current_session_create_info();

        let (session, waiter, stream) =
            SessionData::new(&self.instance, self.system_id, origin, Some(&info))
                .expect("Failed to initalize new session");

        comp.init_frame_controller(&session, waiter, stream);

        if let Some(input) = self.input.get() {
            input.post_session_restart(&session);
        }

        *session_guard = ManuallyDrop::new(session);
    }

    pub fn set_tracking_space(&self, space: vr::ETrackingUniverseOrigin) {
        self.session_data.0.write().unwrap().current_origin = space;
    }

    pub fn get_tracking_space(&self) -> vr::ETrackingUniverseOrigin {
        self.session_data.get().current_origin
    }

    fn end_session(&self) {
        self.session_data.get().session.request_exit().unwrap();
        let mut state = self.session_data.get().state;
        while state != xr::SessionState::STOPPING {
            self.poll_events();
            state = self.session_data.get().state;
        }
        self.session_data.get().session.end().unwrap();
        while state != xr::SessionState::EXITING {
            self.poll_events();
            state = self.session_data.get().state;
        }
    }
}

pub struct AtomicXrTime(AtomicI64);

impl AtomicXrTime {
    #[inline]
    pub fn set(&self, time: xr::Time) {
        self.0.store(time.as_nanos(), Ordering::Relaxed);
    }

    #[inline]
    pub fn get(&self) -> xr::Time {
        xr::Time::from_nanos(self.0.load(Ordering::Relaxed))
    }
}

pub struct SessionReadGuard(RwLock<ManuallyDrop<SessionData>>);
impl SessionReadGuard {
    pub fn get(&self) -> std::sync::RwLockReadGuard<'_, ManuallyDrop<SessionData>> {
        self.0.read().unwrap()
    }

    #[cfg(test)]
    pub fn get_mut(&self) -> std::sync::RwLockWriteGuard<'_, ManuallyDrop<SessionData>> {
        self.0.write().unwrap()
    }
}

pub struct SessionData {
    pub session: xr::Session<xr::vulkan::Vulkan>,
    pub state: xr::SessionState,
    pub view_space: xr::Space,
    local_space: xr::Space,
    stage_space: xr::Space,
    pub current_origin: vr::ETrackingUniverseOrigin,

    pub input_data: crate::input::InputSessionData,
    pub comp_data: crate::compositor::CompositorSessionData,
    /// OpenXR requires graphics information before creating a session, but OpenVR clients don't
    /// have to provide that information until they actually submit a frame. Yet, we need some
    /// information only available behind a session (i.e., calling xrLocateViews for
    /// GetProjectionMatrix), so we will create a session with fake graphics info to appease OpenXR,
    /// that will be replaced with a real one after the application calls IVRSystem::Submit.
    /// When we're using the real session, this will be None.
    /// Note that it also important that this comes after all members which internally use a xr::Session
    /// - structs are dropped in declaration order, and if we drop our temporary Vulkan data
    /// before the session, the runtime will likely be very unhappy.
    temp_vulkan: Option<VulkanData>,
}

#[derive(Debug)]
#[allow(dead_code)] // Results aren't used, but they're printed
pub enum SessionCreationError {
    SessionCreationFailed(xr::sys::Result),
    PollEventFailed(xr::sys::Result),
    BeginSessionFailed(xr::sys::Result),
}

impl SessionData {
    fn new(
        instance: &xr::Instance,
        system_id: xr::SystemId,
        current_origin: vr::ETrackingUniverseOrigin,
        create_info: Option<&xr::vulkan::SessionCreateInfo>,
    ) -> Result<(Self, xr::FrameWaiter, xr::FrameStream<xr::vulkan::Vulkan>), SessionCreationError>
    {
        // required to call
        let _ = instance
            .graphics_requirements::<xr::vulkan::Vulkan>(system_id)
            .unwrap();

        let info;
        let (temp_vulkan, info) = if let Some(info) = create_info {
            // Monado seems to (wrongly) give validation errors unless we call this.
            let pd = unsafe { instance.vulkan_graphics_device(system_id, info.instance) }.unwrap();
            assert_eq!(pd, info.physical_device);
            (None, info)
        } else {
            let vk = VulkanData::new_temporary(&instance, system_id);
            info = vk.as_session_create_info();
            (Some(vk), &info)
        };

        let (session, waiter, stream) =
            unsafe { instance.create_session::<xr::vulkan::Vulkan>(system_id, &info) }
                .map_err(|e| SessionCreationError::SessionCreationFailed(e))?;
        info!("New session created!");

        let view_space = session
            .create_reference_space(xr::ReferenceSpaceType::VIEW, xr::Posef::IDENTITY)
            .unwrap();
        let local_space = session
            .create_reference_space(xr::ReferenceSpaceType::LOCAL, xr::Posef::IDENTITY)
            .unwrap();
        let stage_space = session
            .create_reference_space(xr::ReferenceSpaceType::STAGE, xr::Posef::IDENTITY)
            .unwrap();

        let mut buf = xr::EventDataBuffer::new();
        loop {
            if let Some(xr::Event::SessionStateChanged(state)) = instance
                .poll_event(&mut buf)
                .map_err(|e| SessionCreationError::PollEventFailed(e))?
            {
                if state.state() == xr::SessionState::READY {
                    break;
                }
            }
        }

        info!(
            "OpenXR session state changed: {:?}",
            xr::SessionState::READY
        );
        session
            .begin(xr::ViewConfigurationType::PRIMARY_STEREO)
            .map_err(|e| SessionCreationError::BeginSessionFailed(e))?;
        info!("Began OpenXR session.");

        Ok((
            SessionData {
                temp_vulkan,
                session,
                state: xr::SessionState::READY,
                view_space,
                local_space,
                stage_space,
                input_data: Default::default(),
                comp_data: Default::default(),
                current_origin,
            },
            waiter,
            stream,
        ))
    }

    pub fn tracking_space(&self) -> &xr::Space {
        self.get_space_for_origin(self.current_origin)
    }

    pub fn get_space_for_origin(&self, origin: vr::ETrackingUniverseOrigin) -> &xr::Space {
        match origin {
            vr::ETrackingUniverseOrigin::TrackingUniverseSeated => &self.local_space,
            vr::ETrackingUniverseOrigin::TrackingUniverseStanding => &self.stage_space,
            vr::ETrackingUniverseOrigin::TrackingUniverseRawAndUncalibrated => unreachable!(),
        }
    }

    /// Returns true if this session is not using a temporary graphics setup.
    #[inline]
    pub fn is_real_session(&self) -> bool {
        self.temp_vulkan.is_none()
    }
}

pub struct HandInfo {
    s: &'static str,
    connected: AtomicBool,
    pub path: xr::Path,
}

impl HandInfo {
    #[inline]
    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    fn new(instance: &xr::Instance, path: &'static str) -> Self {
        Self {
            s: path,
            connected: false.into(),
            path: instance.string_to_path(path).unwrap(),
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Hand {
    Left = 1,
    Right,
}

impl TryFrom<vr::TrackedDeviceIndex_t> for Hand {
    type Error = ();
    #[inline]
    fn try_from(value: vr::TrackedDeviceIndex_t) -> Result<Self, Self::Error> {
        match value {
            x if x == Hand::Left as u32 => Ok(Hand::Left),
            x if x == Hand::Right as u32 => Ok(Hand::Right),
            _ => Err(()),
        }
    }
}
