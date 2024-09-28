use super::{ActionData, Input};
use crate::{
    openxr_data::OpenXrData,
    vr::{self, IVRInput010_Interface},
};
use openxr as xr;
use std::collections::HashSet;
use std::ffi::CStr;
use std::sync::Arc;
use vr::EVRInputError::*;

static ACTIONS_JSONS_DIR: &'static CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/input_data/\0").as_bytes(),
    )
};

impl std::fmt::Debug for ActionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionData::Bool(_) => f.write_str("InputAction::Bool"),
            ActionData::Vector1 { .. } => f.write_str("InputAction::Float"),
            ActionData::Vector2 { .. } => f.write_str("InputAction::Vector2"),
            ActionData::Pose { .. } => f.write_str("InputAction::Pose"),
            ActionData::Skeleton { .. } => f.write_str("InputAction::Skeleton"),
            ActionData::Haptic(_) => f.write_str("InputAction::Haptic"),
        }
    }
}

pub(super) struct FakeCompositor(crate::vulkan::VulkanData);
impl crate::InterfaceImpl for FakeCompositor {
    fn get_version(_: &CStr) -> Option<Box<dyn FnOnce(&Arc<Self>) -> *mut std::ffi::c_void>> {
        None
    }
    fn supported_versions() -> &'static [&'static CStr] {
        &[]
    }
}
impl crate::openxr_data::Compositor for FakeCompositor {
    fn current_session_create_info(&self) -> openxr::vulkan::SessionCreateInfo {
        self.0.as_session_create_info()
    }
    fn init_frame_controller(
        &self,
        _: &crate::openxr_data::SessionData,
        _: openxr::FrameWaiter,
        _: openxr::FrameStream<openxr::vulkan::Vulkan>,
    ) {
    }
}

pub(super) struct Fixture {
    pub input: Arc<Input<FakeCompositor>>,
    _comp: Arc<FakeCompositor>,
}

pub(super) trait ActionType: xr::ActionTy {
    fn get_xr_action(data: &ActionData) -> Result<xr::sys::Action, String>;
}

macro_rules! impl_action_type {
    ($ty:ty, $err_ty:literal, $pattern:pat => $extract_action:expr) => {
        impl ActionType for $ty {
            fn get_xr_action(data: &ActionData) -> Result<xr::sys::Action, String> {
                match data {
                    $pattern => Ok($extract_action),
                    other => Err(format!("Expected {} action, got {other:?}", $err_ty)),
                }
            }
        }
    };
}

impl_action_type!(bool, "boolean", ActionData::Bool(a) => a.action.as_raw());
impl_action_type!(f32, "vector1", ActionData::Vector1{ action, .. } => action.as_raw());
impl_action_type!(xr::Vector2f, "vector2", ActionData::Vector2{ action, .. } => action.as_raw());
impl_action_type!(xr::Haptic, "haptic", ActionData::Haptic(a) => a.as_raw());
impl_action_type!(xr::Posef, "pose", ActionData::Pose { action, .. } => action.as_raw());

impl Fixture {
    pub fn new() -> Self {
        crate::init_logging();
        let xr = Arc::new(OpenXrData::new(&crate::clientcore::Injector::default()).unwrap());
        let comp = Arc::new(FakeCompositor(crate::vulkan::VulkanData::new_temporary(
            &xr.instance,
            xr.system_id,
        )));
        xr.compositor.set(Arc::downgrade(&comp));
        let ret = Self {
            input: Input::new(xr.clone()).into(),
            _comp: comp,
        };
        xr.input.set(Arc::downgrade(&ret.input));

        ret
    }

    pub fn load_actions(&self, file: &CStr) {
        let path = &[ACTIONS_JSONS_DIR.to_bytes(), file.to_bytes_with_nul()].concat();
        assert_eq!(
            self.input.SetActionManifestPath(path.as_ptr() as _),
            VRInputError_None
        );
    }

    #[track_caller]
    pub fn verify_bindings<T: ActionType>(
        &self,
        interaction_profile: &str,
        action_name: &CStr,
        expected_bindings: impl Into<HashSet<String>>,
    ) {
        let mut expected_bindings = expected_bindings.into();
        let profile = self
            .input
            .openxr
            .instance
            .string_to_path(interaction_profile)
            .unwrap();

        let handle = self.get_action_handle(action_name);
        let action = self.get_action::<T>(handle);

        let bindings = fakexr::get_suggested_bindings(action, profile);

        let mut found_bindings = Vec::new();

        for binding in bindings {
            assert!(
                expected_bindings.remove(binding.as_str()) || found_bindings.contains(&binding),
                concat!(
                    "Unexpected binding {} for {} action {:?}\n",
                    "found bindings: {:#?}\n",
                    "remaining bindings: {:#?}"
                ),
                binding,
                std::any::type_name::<T>(),
                action_name,
                found_bindings,
                expected_bindings,
            );

            found_bindings.push(binding);
        }

        assert!(
            expected_bindings.is_empty(),
            "Missing expected bindings for {} action {action_name:?}: {expected_bindings:#?}",
            std::any::type_name::<T>(),
        );
    }
}

impl Fixture {
    fn get_action_handle(&self, name: &CStr) -> vr::VRActionHandle_t {
        let mut handle = 0;
        assert_eq!(
            self.input.GetActionHandle(name.as_ptr(), &mut handle),
            VRInputError_None
        );
        assert_ne!(handle, 0);
        handle
    }

    fn get_action_set_handle(&self, name: &CStr) -> vr::VRActionSetHandle_t {
        let mut handle = 0;
        assert_eq!(
            self.input.GetActionSetHandle(name.as_ptr(), &mut handle),
            VRInputError_None
        );
        assert_ne!(handle, 0);
        handle
    }

    fn sync(&self, mut active: vr::VRActiveActionSet_t) {
        assert_eq!(
            self.input.UpdateActionState(
                &mut active,
                std::mem::size_of::<vr::VRActiveActionSet_t>() as u32,
                1
            ),
            VRInputError_None
        );
    }

    #[track_caller]
    fn get_action<T: ActionType>(&self, handle: vr::VRActionHandle_t) -> xr::sys::Action {
        let data = self.input.openxr.session_data.get();
        let actions = data
            .input_data
            .get_loaded_actions()
            .expect("Actions aren't loaded");
        let action = actions
            .try_get_action(handle)
            .expect("Couldn't find action for handle");

        T::get_xr_action(action).expect("Couldn't get OpenXR handle for action")
    }

    fn get_bool_state(
        &self,
        handle: vr::VRActionHandle_t,
    ) -> Result<vr::InputDigitalActionData_t, vr::EVRInputError> {
        let mut state = Default::default();
        let err = self.input.GetDigitalActionData(
            handle,
            &mut state,
            std::mem::size_of::<vr::InputDigitalActionData_t>() as u32,
            0,
        );

        if err != VRInputError_None {
            Err(err)
        } else {
            Ok(state)
        }
    }
}

#[test]
fn no_legacy_input_before_session_setup() {
    let fixture = Fixture::new();

    let got_input = fixture.input.get_legacy_controller_state(
        1,
        &mut vr::VRControllerState_t::default(),
        std::mem::size_of::<vr::VRControllerState_t>() as _,
    );
    assert!(!got_input);

    fixture.input.frame_start_update();
    let got_input = fixture.input.get_legacy_controller_state(
        1,
        &mut vr::VRControllerState_t::default(),
        std::mem::size_of::<vr::VRControllerState_t>() as _,
    );
    assert!(!got_input);
}

#[test]
fn legacy_input() {
    let f = Fixture::new();
    f.input.openxr.restart_session();
    f.input.frame_start_update();
    let mut state = vr::VRControllerState_t::default();
    let got_input = f.input.get_legacy_controller_state(
        1,
        &mut state,
        std::mem::size_of::<vr::VRControllerState_t>() as _,
    );
    assert!(got_input);
    let last_packet_num = { state.unPacketNum };
    // The braces around state.ulButtonPressed are to force create a copy, because
    // VRControllerState_t is a packed struct and references to unaligned fields are undefined.
    assert_eq!(
        { state.ulButtonPressed },
        0,
        "Trigger should be inactive ({:b})",
        { state.ulButtonPressed }
    );

    {
        fakexr::set_action_state(
            f.input
                .openxr
                .session_data
                .get()
                .input_data
                .legacy_actions
                .get()
                .unwrap()
                .trigger_click
                .as_raw(),
            fakexr::ActionState::Bool(true),
        );
    }
    let got_input = f.input.get_legacy_controller_state(
        1,
        &mut state,
        std::mem::size_of::<vr::VRControllerState_t>() as _,
    );
    assert!(got_input);
    assert_eq!({ state.unPacketNum }, last_packet_num);
    assert_eq!(
        { state.ulButtonPressed },
        0,
        "Trigger should be inactive ({:b})",
        { state.ulButtonPressed }
    );

    f.input.frame_start_update();
    let got_input = f.input.get_legacy_controller_state(
        1,
        &mut state,
        std::mem::size_of::<vr::VRControllerState_t>() as _,
    );
    assert!(got_input);
    assert_ne!({ state.unPacketNum }, last_packet_num);
    let state = { state.ulButtonPressed };
    let expected = 1 << vr::EVRButtonId::k_EButton_SteamVR_Trigger as u64;
    assert_eq!(
        state, expected,
        "Trigger should be active (got {state:b}, expected {expected:b})",
    );
}

#[test]
fn unknown_handles() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");

    let handle = f.get_action_handle(c"/actions/set1/in/fakeaction");
    let mut state = Default::default();
    assert_ne!(
        f.input.GetDigitalActionData(
            handle,
            &mut state,
            std::mem::size_of::<vr::InputDigitalActionData_t>() as u32,
            0
        ),
        VRInputError_None
    );
}

#[test]
fn handles_dont_change_after_load() {
    let f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    f.load_actions(c"actions.json");

    let set_load = f.get_action_set_handle(c"/actions/set1");
    assert_eq!(set_load, set1);
    let act_load = f.get_action_handle(c"/actions/set1/in/boolact");
    assert_eq!(act_load, boolact);
}

#[test]
fn input_state_flow() {
    let f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    f.load_actions(c"actions.json");

    assert!(f
        .input
        .openxr
        .session_data
        .get()
        .input_data
        .legacy_actions
        .get()
        .is_some());

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert_eq!(state.bState, false);
    assert_eq!(state.bActive, true);
    assert_eq!(state.bChanged, false);

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert_eq!(state.bState, false);
    assert_eq!(state.bActive, true);
    assert_eq!(state.bChanged, false);

    fakexr::set_action_state(
        f.get_action::<bool>(boolact),
        fakexr::ActionState::Bool(true),
    );

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert_eq!(state.bState, true);
    assert_eq!(state.bActive, true);
    assert_eq!(state.bChanged, true);
}

#[test]
fn reload_manifest_on_session_restart() {
    let f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    f.load_actions(c"actions.json");
    f.input.openxr.restart_session();

    fakexr::set_action_state(
        f.get_action::<bool>(boolact),
        fakexr::ActionState::Bool(true),
    );
    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert_eq!(state.bState, true);
    assert_eq!(state.bActive, true);
}

macro_rules! get_dpad_action {
    ($fixture:expr, $handle:expr, $dpad_data:ident) => {
        let data = $fixture.input.openxr.session_data.get();
        let actions = data.input_data.get_loaded_actions().unwrap();
        let super::ActionData::Bool(super::BoolActionData { dpad_data, .. }) =
            actions.try_get_action($handle).unwrap()
        else {
            panic!("should be bool action");
        };

        let $dpad_data = dpad_data.as_ref().unwrap();
    };
}

#[test]
fn dpad_input() {
    let f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    f.load_actions(c"actions_dpad.json");
    f.input.openxr.restart_session();

    get_dpad_action!(f, boolact, dpad_data);

    fakexr::set_action_state(
        dpad_data.parent.as_raw(),
        fakexr::ActionState::Vector2(0.0, 0.5),
    );
    fakexr::set_action_state(
        dpad_data.click_or_touch.as_ref().unwrap().as_raw(),
        fakexr::ActionState::Bool(true),
    );

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert_eq!(state.bActive, true);
    assert_eq!(state.bState, true);
    assert_eq!(state.bChanged, true);

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert_eq!(state.bActive, true);
    assert_eq!(state.bState, true);
    assert_eq!(state.bChanged, false);

    fakexr::set_action_state(
        dpad_data.parent.as_raw(),
        fakexr::ActionState::Vector2(0.5, 0.0),
    );
    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert_eq!(state.bActive, true);
    assert_eq!(state.bState, false);
    assert_eq!(state.bChanged, true);
}

#[test]
fn dpad_input_different_sets_have_different_actions() {
    let f = Fixture::new();

    let boolact_set1 = f.get_action_handle(c"/actions/set1/in/boolact");
    let boolact_set2 = f.get_action_handle(c"/actions/set2/in/boolact");

    f.load_actions(c"actions_dpad.json");

    get_dpad_action!(f, boolact_set1, set1_dpad);
    get_dpad_action!(f, boolact_set2, set2_dpad);

    assert_ne!(set1_dpad.parent.as_raw(), set2_dpad.parent.as_raw());
}
