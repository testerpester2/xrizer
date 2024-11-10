pub mod vulkan;
use crossbeam_utils::atomic::AtomicCell;
use openxr_sys as xr;
use paste::paste;
use slotmap::{DefaultKey, Key, KeyData, SlotMap};
use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc, Arc, LazyLock, Mutex, MutexGuard, OnceLock, RwLock, Weak,
};

#[derive(Clone, Copy, PartialEq)]
pub enum ActionState {
    Bool(bool),
    Pose,
    Float(f32),
    Vector2(f32, f32),
    Haptic,
}

pub fn set_action_state(action: xr::Action, state: ActionState) {
    let action = xr::Action::to_handle(action).unwrap();
    let s = action.state.load();
    assert_eq!(std::mem::discriminant(&state), std::mem::discriminant(&s));
    action.pending_state.store(Some(state));
}

#[track_caller]
pub fn get_suggested_bindings(action: xr::Action, profile: xr::Path) -> Vec<String> {
    let action = xr::Action::to_handle(action).unwrap();
    let instance = action.instance.upgrade().unwrap();
    let suggested = action.suggested.lock().unwrap();

    suggested
        .get(&profile)
        .unwrap_or_else(|| {
            panic!(
                "No suggested bindings for profile {} for action {:?}",
                instance.get_path_value(profile).unwrap(),
                action.name,
            )
        })
        .iter()
        .map(|path| instance.get_path_value(*path).unwrap())
        .collect()
}

macro_rules! fn_unimplemented_impl {
    ($($param:ident),+) => {
        fn_unimplemented_impl!($($param),+  -> []);
    };
    ($param:ident $(,$rest:ident)* -> [$($params:ident),*]) => {
        paste! {
            #[allow(dead_code)]
            trait [<FnUnimplemented $param>]<$($params,)* $param> {
                extern "system" fn unimplemented($(_: $params,)* _: $param) -> xr::Result {
                    unimplemented!()
                }
            }

            impl<$($params,)* $param> [<FnUnimplemented $param>]<$($params,)* $param> for unsafe extern "system" fn($($params,)* $param) -> xr::Result {}
        }

        fn_unimplemented_impl!($($rest),* -> [$($params,)* $param]);
    };
    (-> [$($params:ident),+]) => {}
}

fn_unimplemented_impl!(A, B, C, D, E, F);

pub extern "system" fn get_instance_proc_addr(
    instance: xr::Instance,
    name: *const c_char,
    function: *mut Option<xr::pfn::VoidFunction>,
) -> xr::Result {
    let name = unsafe { CStr::from_ptr(name) };

    /// Generates match arms for supported functions.
    /// Functions in parenthesis are returned as unimplemented functions - they should be
    /// implemented if a test needs it.
    macro_rules! get_fn {
        ([$($func:tt),+] $pat:pat => $expr:expr) => {
            get_fn!(@arm [$($func),+] -> [] {$pat => $expr})
        };
        (@arm [$name:ident $(,$rest:tt)*] -> [$($arms:tt),*] {$pat:pat => $expr:expr}) => {
            get_fn!(
                @arm
                [$($rest),*] ->
                [
                    $($arms,)*
                    [
                        x if x == const {
                            CStr::from_bytes_with_nul_unchecked(concat!("xr", stringify!($name), "\0").as_bytes())
                        } => Some(std::mem::transmute( paste! { [<$name:snake>] as xr::pfn::$name }))
                    ]
                ]
                {$pat => $expr}
            )
        };
        (@arm [($name:ident) $(,$rest:tt)*] -> [$($arms:tt),*] {$pat:pat => $expr:expr}) => {
            get_fn!(
                @arm
                [$($rest),*] ->
                [
                    $($arms,)*
                    [
                        x if x == const {
                            CStr::from_bytes_with_nul_unchecked(concat!("xr", stringify!($name), "\0").as_bytes())
                        } => Some(std::mem::transmute(xr::pfn::$name::unimplemented as xr::pfn::$name))
                    ]
                ]
                {$pat => $expr}
            )
        };
        (@arm []-> [$([$($arms:tt)*]),+] {$pat:pat => $expr:expr}) => {
            match name {
                $($($arms)*,)+
                $pat => $expr
            }
        }
    }

    if instance == xr::Instance::NULL {
        unsafe {
            *function = get_fn!([CreateInstance, EnumerateInstanceExtensionProperties, (EnumerateApiLayerProperties)]
                other => {
                    println!("unknown func without instance: {other:?}");
                    return xr::Result::ERROR_HANDLE_INVALID;
                }
            );
        }
    } else {
        use vulkan::xr::*;

        unsafe {
            *function = get_fn![[
                GetInstanceProcAddr,
                CreateInstance,
                DestroyInstance,
                (EnumerateInstanceExtensionProperties),
                (EnumerateApiLayerProperties),
                GetVulkanInstanceExtensionsKHR,
                GetVulkanDeviceExtensionsKHR,
                GetVulkanGraphicsDeviceKHR,
                GetVulkanGraphicsRequirementsKHR,
                GetSystem,
                CreateSession,
                DestroySession,
                BeginSession,
                EndSession,
                CreateReferenceSpace,
                PollEvent,
                DestroySpace,
                (LocateViews),
                RequestExitSession,
                (ResultToString),
                (StructureTypeToString),
                (GetInstanceProperties),
                (GetSystemProperties),
                CreateSwapchain,
                DestroySwapchain,
                EnumerateSwapchainImages,
                AcquireSwapchainImage,
                WaitSwapchainImage,
                ReleaseSwapchainImage,
                (EnumerateSwapchainFormats),
                (EnumerateReferenceSpaces),
                CreateActionSpace,
                LocateSpace,
                (EnumerateViewConfigurations),
                (EnumerateEnvironmentBlendModes),
                (GetViewConfigurationProperties),
                (EnumerateViewConfigurationViews),
                BeginFrame,
                EndFrame,
                WaitFrame,
                (ApplyHapticFeedback),
                (StopHapticFeedback),
                (PollEvent),
                StringToPath,
                (PathToString),
                (GetReferenceSpaceBoundsRect),
                GetActionStateBoolean,
                GetActionStateFloat,
                GetActionStateVector2f,
                (GetActionStatePose),
                CreateActionSet,
                DestroyActionSet,
                CreateAction,
                DestroyAction,
                SuggestInteractionProfileBindings,
                AttachSessionActionSets,
                (GetCurrentInteractionProfile),
                SyncActions,
                (EnumerateBoundSourcesForAction),
                (GetInputSourceLocalizedName)
                ]

                other => {
                    println!("unknown func: {other:?}");
                    return xr::Result::ERROR_FUNCTION_UNSUPPORTED;
                }
            ]
        }
    }

    xr::Result::SUCCESS
}

extern "system" fn enumerate_instance_extension_properties(
    layer_name: *const c_char,
    property_capacity_input: u32,
    property_count_output: *mut u32,
    properties: *mut xr::ExtensionProperties,
) -> xr::Result {
    assert!(layer_name.is_null());
    unsafe { *property_count_output = 1 };
    if property_capacity_input > 0 {
        let props =
            unsafe { std::slice::from_raw_parts_mut(properties, property_capacity_input as usize) };
        props[0] = xr::ExtensionProperties {
            ty: xr::ExtensionProperties::TYPE,
            next: std::ptr::null_mut(),
            extension_name: [0 as c_char; xr::MAX_EXTENSION_NAME_SIZE],
            extension_version: 1,
        };
        let name = xr::KHR_VULKAN_ENABLE_EXTENSION_NAME;
        let name =
            unsafe { std::slice::from_raw_parts(name.as_ptr() as *const c_char, name.len()) };
        props[0].extension_name[..name.len()].copy_from_slice(name);
    }
    xr::Result::SUCCESS
}

trait Handle: 'static {
    type XrType: XrType;
    fn instances() -> MutexGuard<'static, SlotMap<DefaultKey, Arc<Self>>>;
    fn to_xr(self: Arc<Self>) -> Self::XrType;
}

trait XrType {
    type Handle: Handle;
    const TO_RAW: fn(Self) -> u64;
    fn to_handle(xr: Self) -> Option<Arc<Self::Handle>>;
}

macro_rules! get_handle {
    ($handle:expr) => {{
        match <_ as XrType>::to_handle($handle) {
            Some(handle) => handle,
            None => {
                return xr::Result::ERROR_HANDLE_INVALID;
            }
        }
    }};
}

macro_rules! impl_handle {
    ($ty:ty, $xr_type:ty) => {
        impl XrType for $xr_type {
            type Handle = $ty;
            const TO_RAW: fn(Self) -> u64 = <$xr_type>::into_raw;
            fn to_handle(xr: $xr_type) -> Option<Arc<Self::Handle>> {
                Self::Handle::instances()
                    .get(DefaultKey::from(KeyData::from_ffi(xr.into_raw())))
                    .map(|i| Arc::clone(i))
            }
        }
        impl Handle for $ty {
            type XrType = $xr_type;
            fn instances() -> MutexGuard<'static, SlotMap<DefaultKey, Arc<Self>>> {
                static I: LazyLock<Mutex<SlotMap<DefaultKey, Arc<$ty>>>> =
                    LazyLock::new(|| Mutex::default());
                I.lock().unwrap()
            }
            fn to_xr(self: Arc<Self>) -> $xr_type {
                let key = Self::instances().insert(self);
                <$xr_type>::from_raw(key.data().as_ffi())
            }
        }
    };
}

struct EventDataBuffer(Vec<u8>);

struct Instance {
    event_receiver: Mutex<mpsc::Receiver<EventDataBuffer>>,
    event_sender: mpsc::Sender<EventDataBuffer>,
    paths: Mutex<SlotMap<DefaultKey, String>>,
    string_to_path: Mutex<HashMap<String, DefaultKey>>,
}

impl Instance {
    fn get_path_value(&self, path: xr::Path) -> Option<String> {
        let key = DefaultKey::from(KeyData::from_ffi(path.into_raw()));
        self.paths.lock().unwrap().get(key).cloned()
    }
}

struct Session {
    instance: Weak<Instance>,
    event_sender: mpsc::Sender<EventDataBuffer>,
    vk_device: AtomicU64,
    attached_sets: OnceLock<Box<[xr::ActionSet]>>,
}

struct ActionSet {
    instance: Weak<Instance>,
    pending_actions: RwLock<Vec<Arc<Action>>>,
    actions: OnceLock<Vec<Arc<Action>>>,
    active: AtomicBool,
}
impl ActionSet {
    fn make_immutable(&self) {
        let actions = std::mem::take(&mut *self.pending_actions.write().unwrap());
        self.actions
            .set(actions)
            .unwrap_or_else(|_| panic!("Action set already immutable"));
    }
}

struct Action {
    instance: Weak<Instance>,
    name: CString,
    localized_name: CString,
    state: AtomicCell<ActionState>,
    changed: AtomicBool,
    pending_state: AtomicCell<Option<ActionState>>,
    suggested: Mutex<HashMap<xr::Path, Vec<xr::Path>>>,
}

impl_handle!(Instance, xr::Instance);
impl_handle!(Session, xr::Session);
impl_handle!(ActionSet, xr::ActionSet);
impl_handle!(Action, xr::Action);

fn destroy_handle<T: XrType>(xr: T) -> xr::Result {
    T::Handle::instances().remove(DefaultKey::from(KeyData::from_ffi(T::TO_RAW(xr))));
    xr::Result::SUCCESS
}

extern "system" fn create_instance(
    _info: *const xr::InstanceCreateInfo,
    instance: *mut xr::Instance,
) -> xr::Result {
    let (tx, rx) = mpsc::channel();
    let inst = Arc::new(Instance {
        event_receiver: rx.into(),
        event_sender: tx,
        paths: Default::default(),
        string_to_path: Default::default(),
    });
    unsafe {
        *instance = inst.to_xr();
    }
    xr::Result::SUCCESS
}

extern "system" fn destroy_instance(instance: xr::Instance) -> xr::Result {
    destroy_handle(instance)
}

extern "system" fn create_session(
    instance: xr::Instance,
    create_info: *const xr::SessionCreateInfo,
    session: *mut xr::Session,
) -> xr::Result {
    let instance = get_handle!(instance);
    let info = unsafe { create_info.as_ref().unwrap() };
    let vk = unsafe {
        (info.next as *const xr::GraphicsBindingVulkanKHR)
            .as_ref()
            .unwrap()
    };
    assert_eq!(vk.ty, xr::GraphicsBindingVulkanKHR::TYPE);
    let sess = Arc::new(Session {
        instance: Arc::downgrade(&instance),
        event_sender: instance.event_sender.clone(),
        vk_device: (vk.device as u64).into(),
        attached_sets: OnceLock::new(),
    });

    let tx = sess.event_sender.clone();
    unsafe {
        *session = sess.to_xr();
    }

    send_event(
        &tx,
        xr::EventDataSessionStateChanged {
            ty: xr::EventDataSessionStateChanged::TYPE,
            next: std::ptr::null(),
            session: unsafe { *session },
            state: xr::SessionState::READY,
            time: xr::Time::from_nanos(0),
        },
    );

    xr::Result::SUCCESS
}

extern "system" fn destroy_session(session: xr::Session) -> xr::Result {
    let s = get_handle!(session);
    // Our Vulkan device needs to still exist when we destroy the session - a real runtime will use
    // it!
    let device = s.vk_device.load(Ordering::Relaxed);
    if !vulkan::Device::validate(device) {
        panic!("Vulkan device invalid ({device})")
    }
    destroy_handle(session);

    xr::Result::SUCCESS
}

extern "system" fn create_action_set(
    instance: xr::Instance,
    _: *const xr::ActionSetCreateInfo,
    set: *mut xr::ActionSet,
) -> xr::Result {
    let instance = get_handle!(instance);
    let s = Arc::new(ActionSet {
        instance: Arc::downgrade(&instance),
        actions: OnceLock::new(),
        pending_actions: RwLock::default(),
        active: false.into(),
    });

    unsafe {
        *set = s.to_xr();
    }
    xr::Result::SUCCESS
}

extern "system" fn destroy_action_set(set: xr::ActionSet) -> xr::Result {
    destroy_handle(set)
}

extern "system" fn create_action(
    set: xr::ActionSet,
    info: *const xr::ActionCreateInfo,
    action: *mut xr::Action,
) -> xr::Result {
    let set = get_handle!(set);
    if set.actions.get().is_some() {
        return xr::Result::ERROR_ACTIONSETS_ALREADY_ATTACHED;
    }

    let info = unsafe { info.as_ref().unwrap() };
    let name = CStr::from_bytes_until_nul(unsafe {
        std::slice::from_raw_parts(info.action_name.as_ptr() as _, info.action_name.len())
    })
    .unwrap();
    for b in name.to_bytes().iter().copied() {
        if !b.is_ascii_alphanumeric() && b != b'-' && b != b'.' && b != b'_' {
            println!(
                "bad character ({:?}) in action name {name:?}",
                std::str::from_utf8(&[b])
            );
            return xr::Result::ERROR_PATH_FORMAT_INVALID;
        }
    }
    let localized_name = CStr::from_bytes_until_nul(unsafe {
        std::slice::from_raw_parts(
            info.localized_action_name.as_ptr() as _,
            info.localized_action_name.len(),
        )
    })
    .unwrap();

    for action in set.pending_actions.read().unwrap().iter() {
        if action.name.as_c_str() == name {
            return xr::Result::ERROR_NAME_DUPLICATED;
        }
        if action.localized_name.as_c_str() == localized_name {
            return xr::Result::ERROR_LOCALIZED_NAME_DUPLICATED;
        }
    }

    let a = Arc::new(Action {
        instance: set.instance.clone(),
        name: name.to_owned(),
        changed: false.into(),
        localized_name: CStr::from_bytes_until_nul(unsafe {
            std::slice::from_raw_parts(
                info.localized_action_name.as_ptr() as _,
                info.localized_action_name.len(),
            )
        })
        .unwrap()
        .to_owned(),
        state: match info.action_type {
            xr::ActionType::BOOLEAN_INPUT => ActionState::Bool(false),
            xr::ActionType::POSE_INPUT => ActionState::Pose,
            xr::ActionType::FLOAT_INPUT => ActionState::Float(0.0),
            xr::ActionType::VECTOR2F_INPUT => ActionState::Vector2(0.0, 0.0),
            xr::ActionType::VIBRATION_OUTPUT => ActionState::Haptic,
            other => unimplemented!("unhandled action type: {other:?}"),
        }
        .into(),
        pending_state: None.into(),
        suggested: Mutex::default(),
    });

    set.pending_actions.write().unwrap().push(a.clone());
    unsafe {
        *action = a.to_xr();
    }
    xr::Result::SUCCESS
}

extern "system" fn destroy_action(action: xr::Action) -> xr::Result {
    destroy_handle(action)
}

extern "system" fn create_action_space(
    _: xr::Session,
    _: *const xr::ActionSpaceCreateInfo,
    _: *mut xr::Space,
) -> xr::Result {
    xr::Result::SUCCESS
}

extern "system" fn get_system(
    _: xr::Instance,
    _: *const xr::SystemGetInfo,
    system_id: *mut xr::SystemId,
) -> xr::Result {
    unsafe { *system_id = xr::SystemId::from_raw(1) };
    xr::Result::SUCCESS
}

fn send_event<T: Copy>(tx: &mpsc::Sender<EventDataBuffer>, event: T) {
    const {
        assert!(std::mem::size_of::<T>() <= std::mem::size_of::<xr::EventDataBuffer>());
    }

    let s = unsafe {
        std::slice::from_raw_parts(&event as *const T as *const u8, std::mem::size_of::<T>())
    }
    .to_vec();
    tx.send(EventDataBuffer(s)).unwrap();
}

extern "system" fn begin_session(_: xr::Session, _: *const xr::SessionBeginInfo) -> xr::Result {
    xr::Result::SUCCESS
}

extern "system" fn destroy_space(_: xr::Space) -> xr::Result {
    xr::Result::SUCCESS
}

extern "system" fn create_reference_space(
    _: xr::Session,
    create_info: *const xr::ReferenceSpaceCreateInfo,
    space: *mut xr::Space,
) -> xr::Result {
    let info = unsafe { create_info.as_ref().unwrap() };
    let val = match info.reference_space_type {
        xr::ReferenceSpaceType::VIEW => 1,
        xr::ReferenceSpaceType::LOCAL => 2,
        xr::ReferenceSpaceType::STAGE => 3,
        other => panic!("unimplemented reference space type: {other:?}"),
    };

    unsafe {
        *space = xr::Space::from_raw(val);
    }
    xr::Result::SUCCESS
}

extern "system" fn poll_event(
    instance: xr::Instance,
    buffer: *mut xr::EventDataBuffer,
) -> xr::Result {
    let instance = get_handle!(instance);
    let recv = instance.event_receiver.lock().unwrap();
    match recv.try_recv() {
        Ok(event) => {
            unsafe {
                buffer
                    .cast::<u8>()
                    .copy_from(event.0.as_ptr(), event.0.len());
            }
            xr::Result::SUCCESS
        }
        Err(mpsc::TryRecvError::Empty) => xr::Result::EVENT_UNAVAILABLE,
        Err(mpsc::TryRecvError::Disconnected) => unreachable!(),
    }
}

extern "system" fn string_to_path(
    instance: xr::Instance,
    string: *const c_char,
    path: *mut xr::Path,
) -> xr::Result {
    let instance = get_handle!(instance);
    let s = unsafe { CStr::from_ptr(string) }.to_str().unwrap();
    let mut string_to_path = instance.string_to_path.lock().unwrap();
    let key = match string_to_path.get(s) {
        Some(p) => *p,
        None => {
            let mut paths = instance.paths.lock().unwrap();
            let key = paths.insert(s.to_string());
            string_to_path.insert(s.to_string(), key);
            key
        }
    };

    unsafe { path.write(xr::Path::from_raw(key.data().as_ffi())) };

    xr::Result::SUCCESS
}

extern "system" fn request_exit_session(session: xr::Session) -> xr::Result {
    let sess = get_handle!(session);
    send_event(
        &sess.event_sender,
        xr::EventDataSessionStateChanged {
            ty: xr::EventDataSessionStateChanged::TYPE,
            next: std::ptr::null(),
            session,
            state: xr::SessionState::STOPPING,
            time: xr::Time::from_nanos(0),
        },
    );
    xr::Result::SUCCESS
}

extern "system" fn end_session(session: xr::Session) -> xr::Result {
    let sess = get_handle!(session);
    send_event(
        &sess.event_sender,
        xr::EventDataSessionStateChanged {
            ty: xr::EventDataSessionStateChanged::TYPE,
            next: std::ptr::null(),
            session,
            state: xr::SessionState::EXITING,
            time: xr::Time::from_nanos(0),
        },
    );
    xr::Result::SUCCESS
}

extern "system" fn suggest_interaction_profile_bindings(
    instance: xr::Instance,
    binding: *const xr::InteractionProfileSuggestedBinding,
) -> xr::Result {
    let _ = get_handle!(instance);
    let binding = unsafe { binding.as_ref().unwrap() };

    let profile_path = binding.interaction_profile;
    let bindings = unsafe {
        std::slice::from_raw_parts(
            binding.suggested_bindings,
            binding.count_suggested_bindings as usize,
        )
    };

    for xr::ActionSuggestedBinding { action, binding } in bindings.iter().copied() {
        let action = get_handle!(action);
        action
            .suggested
            .lock()
            .unwrap()
            .entry(profile_path.clone())
            .or_default()
            .push(binding);
    }

    xr::Result::SUCCESS
}

extern "system" fn attach_session_action_sets(
    session: xr::Session,
    info: *const xr::SessionActionSetsAttachInfo,
) -> xr::Result {
    let sesh = get_handle!(session);
    let sets =
        unsafe { std::slice::from_raw_parts((*info).action_sets, (*info).count_action_sets as _) };
    if sesh.attached_sets.set(sets.into()).is_ok() {
        // make action sets immutable
        for set in sesh.attached_sets.get().unwrap() {
            let set = get_handle!(*set);
            set.make_immutable();
        }
        xr::Result::SUCCESS
    } else {
        xr::Result::ERROR_ACTIONSETS_ALREADY_ATTACHED
    }
}

extern "system" fn sync_actions(
    session: xr::Session,
    info: *const xr::ActionsSyncInfo,
) -> xr::Result {
    let session = get_handle!(session);
    let Some(attached) = session.attached_sets.get() else {
        return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED;
    };
    for set in attached {
        let set = get_handle!(*set);
        set.active.store(false, Ordering::Relaxed);
    }
    let sets = unsafe {
        std::slice::from_raw_parts(
            (*info).active_action_sets,
            (*info).count_active_action_sets as _,
        )
    };
    for set in sets {
        if !attached.contains(&set.action_set) {
            return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED;
        }
        let set = get_handle!(set.action_set);
        let Some(actions) = set.actions.get() else {
            return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED;
        };
        set.active.store(true, Ordering::Relaxed);

        for action in actions {
            match action.pending_state.take() {
                Some(new_state) => {
                    let old_state = action.state.load();
                    if old_state != new_state {
                        action.changed.store(true, Ordering::Relaxed);
                        action.state.store(new_state);
                    }
                }
                None => {
                    action.changed.store(false, Ordering::Relaxed);
                }
            }
        }
    }
    xr::Result::SUCCESS
}

fn get_action_if_attached(
    session: &Session,
    info: *const xr::ActionStateGetInfo,
) -> Option<(Arc<ActionSet>, Arc<Action>)> {
    let sets = session.attached_sets.get()?;
    let action = xr::Action::to_handle(unsafe { (*info).action })?;
    sets.into_iter().find_map(|set| {
        let set = xr::ActionSet::to_handle(*set)?;
        for a in set.actions.get().unwrap() {
            if Arc::as_ptr(a) == Arc::as_ptr(&action) {
                return Some((set, action.clone()));
            }
        }
        None
    })
}

extern "system" fn get_action_state_boolean(
    session: xr::Session,
    info: *const xr::ActionStateGetInfo,
    state: *mut xr::ActionStateBoolean,
) -> xr::Result {
    let session = get_handle!(session);
    let Some((set, action)) = get_action_if_attached(&session, info) else {
        return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED;
    };

    let ActionState::Bool(b) = action.state.load() else {
        return xr::Result::ERROR_ACTION_TYPE_MISMATCH;
    };
    let state = unsafe { state.as_mut().unwrap() };
    if set.active.load(Ordering::Relaxed) {
        state.is_active = true.into();
        state.current_state = b.into();
        state.changed_since_last_sync = action.changed.load(Ordering::Relaxed).into();
    } else {
        state.is_active = false.into();
        state.current_state = false.into();
    }
    xr::Result::SUCCESS
}

extern "system" fn get_action_state_float(
    session: xr::Session,
    info: *const xr::ActionStateGetInfo,
    state: *mut xr::ActionStateFloat,
) -> xr::Result {
    let session = get_handle!(session);
    let Some((set, action)) = get_action_if_attached(&session, info) else {
        return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED;
    };

    let ActionState::Float(f) = action.state.load() else {
        return xr::Result::ERROR_ACTION_TYPE_MISMATCH;
    };
    let state = unsafe { state.as_mut().unwrap() };
    if set.active.load(Ordering::Relaxed) {
        state.is_active = true.into();
        state.current_state = f;
    } else {
        state.is_active = false.into();
        state.current_state = 0.0;
    }
    xr::Result::SUCCESS
}

extern "system" fn get_action_state_vector2f(
    session: xr::Session,
    info: *const xr::ActionStateGetInfo,
    state: *mut xr::ActionStateVector2f,
) -> xr::Result {
    let session = get_handle!(session);
    let Some((set, action)) = get_action_if_attached(&session, info) else {
        return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED;
    };

    let ActionState::Vector2(x, y) = action.state.load() else {
        return xr::Result::ERROR_ACTION_TYPE_MISMATCH;
    };
    let state = unsafe { state.as_mut().unwrap() };
    if set.active.load(Ordering::Relaxed) {
        state.is_active = true.into();
        state.current_state = xr::Vector2f { x, y };
    } else {
        state.is_active = false.into();
        state.current_state = Default::default();
    }

    xr::Result::SUCCESS
}

extern "system" fn locate_space(
    _space: xr::Space,
    _base_space: xr::Space,
    _time: xr::Time,
    _location: *mut xr::SpaceLocation,
) -> xr::Result {
    xr::Result::SUCCESS
}

extern "system" fn create_swapchain(
    _session: xr::Session,
    _info: *const xr::SwapchainCreateInfo,
    _swapchain: *mut xr::Swapchain,
) -> xr::Result {
    xr::Result::SUCCESS
}

extern "system" fn destroy_swapchain(_: xr::Swapchain) -> xr::Result {
    xr::Result::SUCCESS
}

extern "system" fn enumerate_swapchain_images(
    _swapchain: xr::Swapchain,
    _: u32,
    output: *mut u32,
    _: *mut xr::SwapchainImageBaseHeader,
) -> xr::Result {
    if let Some(output) = unsafe { output.as_mut() } {
        *output = 0;
    }
    xr::Result::SUCCESS
}

extern "system" fn acquire_swapchain_image(
    _swapchain: xr::Swapchain,
    _info: *const xr::SwapchainImageAcquireInfo,
    _index: *mut u32,
) -> xr::Result {
    xr::Result::SUCCESS
}

extern "system" fn wait_swapchain_image(
    _swapchain: xr::Swapchain,
    _info: *const xr::SwapchainImageWaitInfo,
) -> xr::Result {
    xr::Result::SUCCESS
}

extern "system" fn release_swapchain_image(
    _swapchain: xr::Swapchain,
    _info: *const xr::SwapchainImageReleaseInfo,
) -> xr::Result {
    xr::Result::SUCCESS
}

extern "system" fn wait_frame(
    _session: xr::Session,
    _info: *const xr::FrameWaitInfo,
    state: *mut xr::FrameState,
) -> xr::Result {
    unsafe {
        state.write(xr::FrameState {
            ty: xr::FrameState::TYPE,
            next: std::ptr::null_mut(),
            predicted_display_time: xr::Time::from_nanos(1),
            predicted_display_period: xr::Duration::from_nanos(1),
            should_render: false.into(),
        })
    }
    xr::Result::SUCCESS
}

extern "system" fn begin_frame(
    _session: xr::Session,
    _info: *const xr::FrameBeginInfo,
) -> xr::Result {
    xr::Result::SUCCESS
}

extern "system" fn end_frame(_session: xr::Session, _info: *const xr::FrameEndInfo) -> xr::Result {
    xr::Result::SUCCESS
}
