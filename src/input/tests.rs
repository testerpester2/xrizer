use super::{Input, InputAction};
use crate::{
    openxr_data::OpenXrData,
    vr::{self, IVRInput010_Interface},
};
use openxr as xr;
use std::ffi::CStr;
use std::sync::Arc;
use vr::EVRInputError::*;

static ACTIONS_JSON: &'static CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/input_data/actions.json\0"
        )
        .as_bytes(),
    )
};

impl std::fmt::Debug for InputAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputAction::Bool(_) => f.write_str("InputAction::Bool"),
            InputAction::Float { .. } => f.write_str("InputAction::Float"),
            InputAction::Vector2(_) => f.write_str("InputAction::Vector2"),
            InputAction::Pose { .. } => f.write_str("InputAction::Pose"),
            InputAction::Skeleton { .. } => f.write_str("InputAction::Skeleton"),
        }
    }
}

struct FakeCompositor(crate::vulkan::VulkanData);
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

struct Fixture {
    input: Arc<Input<FakeCompositor>>,
    _comp: Arc<FakeCompositor>,
}

impl Fixture {
    fn new() -> Self {
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

    fn load_actions(&self) {
        assert_eq!(
            self.input.SetActionManifestPath(ACTIONS_JSON.as_ptr()),
            VRInputError_None
        );
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
    fn get_bool_action(&self, handle: vr::VRActionHandle_t) -> xr::sys::Action {
        let data = self.input.openxr.session_data.get();
        let actions = data
            .input_data
            .get_loaded_actions()
            .expect("Actions aren't loaded");
        let action = match actions
            .try_get_action(handle)
            .expect("Couldn't find action for handle")
        {
            InputAction::Bool(a) => a,
            other => panic!("Expected boolean action, got {other:?}"),
        };
        action.as_raw()
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
    f.load_actions();

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

    f.load_actions();

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

    f.load_actions();
    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let mut state = vr::InputDigitalActionData_t::default();
    assert_eq!(
        f.input.GetDigitalActionData(
            boolact,
            &mut state,
            std::mem::size_of::<vr::InputDigitalActionData_t>() as u32,
            0
        ),
        VRInputError_None
    );
    assert_eq!(state.bState, false);

    fakexr::set_action_state(f.get_bool_action(boolact), fakexr::ActionState::Bool(true));

    assert_eq!(
        f.input.GetDigitalActionData(
            boolact,
            &mut state,
            std::mem::size_of::<vr::InputDigitalActionData_t>() as u32,
            0
        ),
        VRInputError_None
    );
    assert_eq!(state.bState, false);

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });
    assert_eq!(
        f.input.GetDigitalActionData(
            boolact,
            &mut state,
            std::mem::size_of::<vr::InputDigitalActionData_t>() as u32,
            0
        ),
        VRInputError_None
    );
    assert_eq!(state.bState, true);
}

#[test]
fn reload_manifest_on_session_restart() {
    let f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    f.load_actions();
    f.input.openxr.restart_session();

    fakexr::set_action_state(f.get_bool_action(boolact), fakexr::ActionState::Bool(true));
    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let mut state = Default::default();
    assert_eq!(
        f.input
            .GetDigitalActionData(boolact, &mut state, std::mem::size_of_val(&state) as u32, 0),
        VRInputError_None
    );
    assert_eq!(state.bState, true);
    assert_eq!(state.bActive, true);
}
