pub mod vulkan;
use crossbeam_utils::atomic::AtomicCell;
use openxr_sys as xr;
use paste::paste;
use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc, Arc, Mutex, OnceLock, RwLock, Weak,
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
    let action = Action::from_xr(action);
    let s = action.state.load();
    assert_eq!(std::mem::discriminant(&state), std::mem::discriminant(&s));
    action.pending_state.store(Some(state));
}

#[track_caller]
pub fn get_suggested_bindings(action: xr::Action, profile: xr::Path) -> Vec<String> {
    let action = Action::from_xr(action);
    let path = Path::from_xr(profile);
    let suggested = action.suggested.lock().unwrap();

    suggested
        .get(&path)
        .unwrap_or_else(|| {
            panic!(
                "No suggested bindings for profile {} for action {:?}",
                path.val, action.name,
            )
        })
        .iter()
        .map(|path| path.val.clone())
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
        let instance = instance.into_raw() as *const Instance;
        if unsafe { (*instance).debug } != Instance::DEBUG_VAL {
            return xr::Result::ERROR_HANDLE_INVALID;
        }

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
            unsafe { std::slice::from_raw_parts_mut(properties, property_count_output as usize) };
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

trait Handle: Sized {
    type XrType;
    const DEBUG_VAL: u64;
    const TO_RAW: fn(Self::XrType) -> u64;

    fn get_debug_val(ptr: *const Self) -> u64;

    fn validate(ptr: *const Self) -> bool {
        Self::get_debug_val(ptr) == Self::DEBUG_VAL
    }
    fn from_xr(xr: Self::XrType) -> Arc<Self> {
        let ptr = Self::TO_RAW(xr) as *const Self;
        assert!(Self::validate(ptr));
        unsafe {
            Arc::increment_strong_count(ptr);
            Arc::from_raw(ptr)
        }
    }
}

macro_rules! impl_handle {
    ($ty:ty, $xr:ty, $debug:literal) => {
        impl Handle for $ty {
            type XrType = $xr;
            const DEBUG_VAL: u64 = $debug;
            const TO_RAW: fn(Self::XrType) -> u64 = <$xr>::into_raw;

            fn get_debug_val(ptr: *const Self) -> u64 {
                unsafe { *(ptr as *const u64) }
            }
        }
    };
}

#[repr(C)]
struct Instance {
    debug: u64,
    event_receiver: mpsc::Receiver<xr::EventDataBuffer>,
    event_sender: mpsc::Sender<xr::EventDataBuffer>,
    paths: Mutex<HashMap<String, Arc<Path>>>,
}

#[repr(C)]
#[derive(Eq, Hash, PartialEq, Debug)]
struct Path {
    debug: u64,
    val: String,
}

#[repr(C)]
struct Session {
    debug: u64,
    instance: Weak<Instance>,
    event_sender: mpsc::Sender<xr::EventDataBuffer>,
    vk_device: AtomicU64,
    attached_sets: OnceLock<Box<[xr::ActionSet]>>,
}

#[repr(C)]
struct ActionSet {
    debug: u64,
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

#[repr(C)]
struct Action {
    debug: u64,
    name: CString,
    localized_name: CString,
    state: AtomicCell<ActionState>,
    changed: AtomicBool,
    pending_state: AtomicCell<Option<ActionState>>,
    suggested: Mutex<HashMap<Arc<Path>, Vec<Arc<Path>>>>,
}

impl_handle!(Instance, xr::Instance, 342);
impl_handle!(Session, xr::Session, 442);
impl_handle!(ActionSet, xr::ActionSet, 542);
impl_handle!(Action, xr::Action, 662);
impl_handle!(Path, xr::Path, 762);

macro_rules! destroy_handle {
    ($ty:ty, $handle:expr) => {{
        unsafe { Arc::from_raw($handle.into_raw() as *const $ty) };

        xr::Result::SUCCESS
    }};
}

extern "system" fn create_instance(
    _info: *const xr::InstanceCreateInfo,
    instance: *mut xr::Instance,
) -> xr::Result {
    let (tx, rx) = mpsc::channel();
    let inst = Arc::new(Instance {
        debug: Instance::DEBUG_VAL,
        event_receiver: rx,
        event_sender: tx,
        paths: Default::default(),
    });
    unsafe {
        *instance = xr::Instance::from_raw(Arc::into_raw(inst) as u64);
    }
    xr::Result::SUCCESS
}

extern "system" fn destroy_instance(instance: xr::Instance) -> xr::Result {
    destroy_handle!(Instance, instance)
}

extern "system" fn create_session(
    instance: xr::Instance,
    create_info: *const xr::SessionCreateInfo,
    session: *mut xr::Session,
) -> xr::Result {
    let instance = Instance::from_xr(instance);
    let info = unsafe { create_info.as_ref().unwrap() };
    let vk = unsafe {
        (info.next as *const xr::GraphicsBindingVulkanKHR)
            .as_ref()
            .unwrap()
    };
    assert_eq!(vk.ty, xr::GraphicsBindingVulkanKHR::TYPE);
    let sess = Arc::new(Session {
        debug: Session::DEBUG_VAL,
        instance: Arc::downgrade(&instance),
        event_sender: instance.event_sender.clone(),
        vk_device: (vk.device as u64).into(),
        attached_sets: OnceLock::new(),
    });

    let tx = sess.event_sender.clone();
    unsafe {
        *session = xr::Session::from_raw(Arc::into_raw(sess) as u64);
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
    let session = unsafe { Arc::from_raw(session.into_raw() as *const Session) };
    // Our Vulkan device needs to still exist when we destroy the session - a real runtime will use
    // it!
    let device = session.vk_device.load(Ordering::Relaxed);
    if !vulkan::Device::validate(device) {
        panic!("Vulkan device invalid ({device})")
    }
    assert_eq!(Arc::strong_count(&session), 1);
    xr::Result::SUCCESS
}

extern "system" fn create_action_set(
    _: xr::Instance,
    _: *const xr::ActionSetCreateInfo,
    set: *mut xr::ActionSet,
) -> xr::Result {
    let s = ActionSet {
        debug: ActionSet::DEBUG_VAL,
        actions: OnceLock::new(),
        pending_actions: RwLock::default(),
        active: false.into(),
    };

    unsafe {
        *set = xr::ActionSet::from_raw(Arc::into_raw(s.into()) as u64);
    }
    xr::Result::SUCCESS
}

extern "system" fn destroy_action_set(set: xr::ActionSet) -> xr::Result {
    destroy_handle!(ActionSet, set)
}

extern "system" fn create_action(
    set: xr::ActionSet,
    info: *const xr::ActionCreateInfo,
    action: *mut xr::Action,
) -> xr::Result {
    let set = ActionSet::from_xr(set);
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
        debug: Action::DEBUG_VAL,
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
        *action = xr::Action::from_raw(Arc::into_raw(a) as u64);
    }
    xr::Result::SUCCESS
}

extern "system" fn destroy_action(action: xr::Action) -> xr::Result {
    destroy_handle!(Action, action)
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

fn send_event<T>(tx: &mpsc::Sender<xr::EventDataBuffer>, event: T) {
    const {
        assert!(std::mem::size_of::<T>() <= std::mem::size_of::<xr::EventDataBuffer>());
    }

    let mut raw_event = xr::EventDataBuffer {
        ty: xr::EventDataBuffer::TYPE,
        next: std::ptr::null(),
        varying: [0; 4000],
    };
    unsafe {
        (&mut raw_event as *mut _ as *mut T).copy_from_nonoverlapping(&event, 1);
    }
    tx.send(raw_event).unwrap();
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
    let instance = Instance::from_xr(instance);
    match instance.event_receiver.try_recv() {
        Ok(event) => {
            unsafe { *buffer = event };
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
    let instance = Instance::from_xr(instance);
    let s = unsafe { CStr::from_ptr(string) }.to_str().unwrap();
    let mut paths = instance.paths.lock().unwrap();
    let p = Arc::clone(paths.entry(s.to_string()).or_insert(Arc::new(Path {
        debug: Path::DEBUG_VAL,
        val: s.to_string(),
    })));
    unsafe { path.write(xr::Path::from_raw(Arc::into_raw(p) as u64)) }
    xr::Result::SUCCESS
}

extern "system" fn request_exit_session(session: xr::Session) -> xr::Result {
    let sess = Session::from_xr(session);
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
    let sess = Session::from_xr(session);
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
    let _ = Instance::from_xr(instance);
    let binding = unsafe { binding.as_ref().unwrap() };

    let profile_path = Path::from_xr(binding.interaction_profile);
    let bindings = unsafe {
        std::slice::from_raw_parts(
            binding.suggested_bindings,
            binding.count_suggested_bindings as usize,
        )
    };

    for xr::ActionSuggestedBinding { action, binding } in bindings.iter().copied() {
        let action = Action::from_xr(action);
        let binding = Path::from_xr(binding);
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
    let sesh = Session::from_xr(session);
    let sets =
        unsafe { std::slice::from_raw_parts((*info).action_sets, (*info).count_action_sets as _) };
    if sesh.attached_sets.set(sets.into()).is_ok() {
        // make action sets immutable
        for set in sesh.attached_sets.get().unwrap() {
            let set = ActionSet::from_xr(*set);
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
    let session = Session::from_xr(session);
    let Some(attached) = session.attached_sets.get() else {
        return xr::Result::ERROR_ACTIONSET_NOT_ATTACHED;
    };
    for set in attached {
        let set = ActionSet::from_xr(*set);
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
        let set = ActionSet::from_xr(set.action_set);
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
    let action = Action::from_xr(unsafe { (*info).action });
    sets.into_iter().find_map(|set| {
        let set = ActionSet::from_xr(*set);
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
    let session = Session::from_xr(session);
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
    let session = Session::from_xr(session);
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
    let session = Session::from_xr(session);
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
        (*state).should_render = false.into();
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
