use super::{
    knuckles::Knuckles, vive_controller::ViveWands, ActionData, Input, InteractionProfile,
};
use crate::{
    openxr_data::OpenXrData,
    vr::{self, IVRInput010_Interface},
};
use fakexr::UserPath::*;
use glam::{Mat3, Quat, Vec3};
use openxr as xr;
use std::collections::HashSet;
use std::f32::consts::FRAC_PI_4;
use std::ffi::CStr;
use std::sync::Arc;

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
impl_action_type!(f32, "vector1", ActionData::Vector1(d) => d.action.as_raw());
impl_action_type!(xr::Vector2f, "vector2", ActionData::Vector2{ action, .. } => action.as_raw());
impl_action_type!(xr::Haptic, "haptic", ActionData::Haptic(a) => a.as_raw());
//impl_action_type!(xr::Posef, "pose", ActionData::Pose { action, .. } => action.as_raw());

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
            vr::EVRInputError::None
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
    pub fn get_action_handle(&self, name: &CStr) -> vr::VRActionHandle_t {
        let mut handle = 0;
        assert_eq!(
            self.input.GetActionHandle(name.as_ptr(), &mut handle),
            vr::EVRInputError::None
        );
        assert_ne!(handle, 0);
        handle
    }

    fn get_action_set_handle(&self, name: &CStr) -> vr::VRActionSetHandle_t {
        let mut handle = 0;
        assert_eq!(
            self.input.GetActionSetHandle(name.as_ptr(), &mut handle),
            vr::EVRInputError::None
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
            vr::EVRInputError::None
        );
    }

    #[track_caller]
    pub fn get_action<T: ActionType>(&self, handle: vr::VRActionHandle_t) -> xr::sys::Action {
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

    fn get_pose(
        &self,
        handle: vr::VRActionHandle_t,
        restrict: vr::VRInputValueHandle_t,
    ) -> Result<vr::InputPoseActionData_t, vr::EVRInputError> {
        let mut state = Default::default();
        let err = self.input.GetPoseActionDataForNextFrame(
            handle,
            vr::ETrackingUniverseOrigin::Seated,
            &mut state,
            std::mem::size_of_val(&state) as u32,
            restrict,
        );

        if err != vr::EVRInputError::None {
            Err(err)
        } else {
            Ok(state)
        }
    }

    fn get_bool_state(
        &self,
        handle: vr::VRActionHandle_t,
    ) -> Result<vr::InputDigitalActionData_t, vr::EVRInputError> {
        self.get_bool_state_hand(handle, 0)
    }

    fn get_bool_state_hand(
        &self,
        handle: vr::VRActionHandle_t,
        restrict: vr::VRInputValueHandle_t,
    ) -> Result<vr::InputDigitalActionData_t, vr::EVRInputError> {
        let mut state = Default::default();
        let err = self.input.GetDigitalActionData(
            handle,
            &mut state,
            std::mem::size_of::<vr::InputDigitalActionData_t>() as u32,
            restrict,
        );

        if err != vr::EVRInputError::None {
            Err(err)
        } else {
            Ok(state)
        }
    }

    fn set_interaction_profile<T: InteractionProfile>(&self, hand: fakexr::UserPath) {
        fakexr::set_interaction_profile(
            self.raw_session(),
            hand,
            self.input
                .openxr
                .instance
                .string_to_path(T::PROFILE_PATH)
                .unwrap(),
        );
        self.input.openxr.poll_events();
    }

    fn raw_session(&self) -> xr::sys::Session {
        self.input.openxr.session_data.get().session.as_raw()
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
            LeftHand,
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
    let expected = 1 << vr::EVRButtonId::SteamVR_Trigger as u64;
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
        vr::EVRInputError::None
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
    assert_eq!(state.bActive, false);
    assert_eq!(state.bChanged, false);

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert_eq!(state.bState, false);
    assert_eq!(state.bActive, false);
    assert_eq!(state.bChanged, false);

    fakexr::set_action_state(
        f.get_action::<bool>(boolact),
        fakexr::ActionState::Bool(true),
        LeftHand,
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
        LeftHand,
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
        LeftHand,
    );
    fakexr::set_action_state(
        dpad_data.click_or_touch.as_ref().unwrap().as_raw(),
        fakexr::ActionState::Bool(true),
        LeftHand,
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
        LeftHand,
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

#[track_caller]
fn compare_pose(expected: xr::Posef, actual: xr::Posef) {
    let epos = expected.position;
    let apos = actual.position;
    assert!(
        (apos.x - epos.x).abs() < f32::EPSILON
            && (apos.y - epos.y).abs() < f32::EPSILON
            && (apos.z - epos.z).abs() < f32::EPSILON,
        "expected position: {epos:?}\nactual position: {apos:?}"
    );

    let erot = expected.orientation;
    let arot = actual.orientation;
    assert!(
        arot.x == erot.x && arot.y == erot.y && arot.z == erot.z && arot.w == erot.w,
        "expected orientation: {erot:?}\nactual orientation: {arot:?}",
    );
}

fn hmdmatrix34_to_pose(mat: vr::HmdMatrix34_t) -> xr::Posef {
    let mat = mat.m;
    let pos = xr::Vector3f {
        x: mat[0][3],
        y: mat[1][3],
        z: mat[2][3],
    };
    let rot = Quat::from_mat3(
        &Mat3::from_cols(
            Vec3::from_slice(&mat[0][..3]),
            Vec3::from_slice(&mat[1][..3]),
            Vec3::from_slice(&mat[2][..3]),
        )
        .transpose(),
    );
    xr::Posef {
        position: pos,
        orientation: xr::Quaternionf {
            x: rot.x,
            y: rot.y,
            z: rot.z,
            w: rot.w,
        },
    }
}

#[test]
fn raw_pose_is_grip_at_aim() {
    let f = Fixture::new();

    let pose_handle = f.get_action_handle(c"/actions/set1/in/pose");
    let left_hand = {
        let mut h = 0;
        assert_eq!(
            f.input
                .GetInputSourceHandle(c"/user/hand/left".as_ptr(), &mut h),
            vr::EVRInputError::None
        );
        h
    };
    f.load_actions(c"actions.json");
    f.set_interaction_profile::<Knuckles>(LeftHand);

    let grip_rot = Quat::from_rotation_x(-FRAC_PI_4);
    let grip = xr::Posef {
        position: xr::Vector3f {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        },
        orientation: xr::Quaternionf {
            x: grip_rot.x,
            y: grip_rot.y,
            z: grip_rot.z,
            w: grip_rot.w,
        },
    };

    fakexr::set_grip(f.raw_session(), LeftHand, grip);

    let aim = xr::Posef {
        position: xr::Vector3f {
            x: 0.7,
            y: 0.6,
            z: 1.0,
        },
        orientation: xr::Quaternionf::IDENTITY,
    };

    fakexr::set_aim(f.raw_session(), LeftHand, aim);

    let data = f.get_pose(pose_handle, left_hand).unwrap();

    assert_eq!(data.bActive, true);
    assert_eq!(data.activeOrigin, left_hand);

    let pose = data.pose;
    assert_eq!(pose.bDeviceIsConnected, true);
    assert_eq!(pose.bPoseIsValid, true);
    assert_eq!(pose.eTrackingResult, vr::ETrackingResult::Running_OK);

    compare_pose(
        xr::Posef {
            position: aim.position,
            orientation: grip.orientation,
        },
        hmdmatrix34_to_pose(pose.mDeviceToAbsoluteTracking),
    );
}

#[test]
fn raw_pose_waitgetposes_and_skeletal_pose_identical() {
    let f = Fixture::new();
    let left_hand = {
        let mut h = 0;
        assert_eq!(
            f.input
                .GetInputSourceHandle(c"/user/hand/left".as_ptr(), &mut h),
            vr::EVRInputError::None
        );
        h
    };
    let pose_handle = f.get_action_handle(c"/actions/set1/in/pose");
    let skel_handle = f.get_action_handle(c"/actions/set1/in/skellyl");
    f.load_actions(c"actions.json");
    f.set_interaction_profile::<Knuckles>(LeftHand);
    let rot = Quat::from_rotation_x(-FRAC_PI_4);
    let pose = xr::Posef {
        position: xr::Vector3f {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        },
        orientation: xr::Quaternionf {
            x: rot.x,
            y: rot.y,
            z: rot.z,
            w: rot.w,
        },
    };
    fakexr::set_grip(f.raw_session(), LeftHand, pose);
    fakexr::set_aim(f.raw_session(), LeftHand, pose);

    let seated_origin = vr::ETrackingUniverseOrigin::Seated;
    let waitgetposes_pose = f
        .input
        .get_controller_pose(super::Hand::Left, Some(seated_origin))
        .expect("WaitGetPoses should succeed");

    let mut raw_pose = vr::InputPoseActionData_t {
        pose: vr::TrackedDevicePose_t {
            eTrackingResult: vr::ETrackingResult::Running_OutOfRange,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut skel_pose = raw_pose;

    let ret = f.input.GetPoseActionDataForNextFrame(
        pose_handle,
        seated_origin,
        &mut raw_pose,
        std::mem::size_of_val(&raw_pose) as u32,
        left_hand,
    );
    assert_eq!(ret, vr::EVRInputError::None);
    compare_pose(
        hmdmatrix34_to_pose(waitgetposes_pose.mDeviceToAbsoluteTracking),
        hmdmatrix34_to_pose(raw_pose.pose.mDeviceToAbsoluteTracking),
    );

    let ret = f.input.GetPoseActionDataForNextFrame(
        skel_handle,
        seated_origin,
        &mut skel_pose,
        std::mem::size_of_val(&skel_pose) as u32,
        0,
    );
    assert_eq!(ret, vr::EVRInputError::None);

    compare_pose(
        hmdmatrix34_to_pose(waitgetposes_pose.mDeviceToAbsoluteTracking),
        hmdmatrix34_to_pose(skel_pose.pose.mDeviceToAbsoluteTracking),
    );
}

#[test]
fn dpad_input_use_non_dpad_when_available() {
    let f = Fixture::new();
    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    f.load_actions(c"actions_dpad_mixed.json");
    f.input.openxr.restart_session();

    get_dpad_action!(f, boolact, _dpad);

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert_eq!(state.bState, false);
    assert_eq!(state.bActive, false);
    assert_eq!(state.bChanged, false);

    fakexr::set_action_state(
        f.get_action::<bool>(boolact),
        fakexr::ActionState::Bool(true),
        LeftHand,
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

macro_rules! get_grab_action {
    ($fixture:expr, $handle:expr, $grab_data:ident) => {
        let data = $fixture.input.openxr.session_data.get();
        let actions = data.input_data.get_loaded_actions().unwrap();
        let super::ActionData::Bool(super::BoolActionData { grab_data, .. }) =
            actions.try_get_action($handle).unwrap()
        else {
            panic!("should be bool action");
        };

        let $grab_data = grab_data.as_ref().unwrap();
    };
}

#[test]
fn grab_binding() {
    let f = Fixture::new();
    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");
    f.load_actions(c"actions.json");
    get_grab_action!(f, boolact, grab_data);

    let value_state_check = |force, value, state, changed, line| {
        fakexr::set_action_state(
            grab_data.force_action.as_raw(),
            fakexr::ActionState::Float(force),
            LeftHand,
        );
        fakexr::set_action_state(
            grab_data.value_action.as_raw(),
            fakexr::ActionState::Float(value),
            LeftHand,
        );
        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });

        let s = f.get_bool_state(boolact).unwrap();
        assert_eq!(s.bState, state, "state failed (line {line})");
        assert_eq!(s.bActive, true, "active failed (line {line})");
        assert_eq!(s.bChanged, changed, "changed failed (line {line})");
    };

    let grab = super::GrabBindingData::GRAB_THRESHOLD;
    let release = super::GrabBindingData::RELEASE_THRESHOLD;
    value_state_check(grab - 0.1, 1.0, false, false, line!());
    value_state_check(grab, 1.0, true, true, line!());
    value_state_check(0.0, 1.0, true, false, line!());
    value_state_check(0.0, release, false, true, line!());
    value_state_check(grab - 0.1, 1.0, false, false, line!());
}

#[test]
fn grab_per_hand() {
    let f = Fixture::new();
    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    let mut left = 0;
    let ret = f
        .input
        .GetInputSourceHandle(c"/user/hand/left".as_ptr(), &mut left);
    assert_eq!(ret, vr::EVRInputError::None);
    let mut right = 0;
    let ret = f
        .input
        .GetInputSourceHandle(c"/user/hand/right".as_ptr(), &mut right);
    assert_eq!(ret, vr::EVRInputError::None);

    f.load_actions(c"actions_dpad_mixed.json");

    get_grab_action!(f, set1, grab_data);

    let value_state_check = |force, value, hand, state, changed, line| {
        fakexr::set_action_state(
            grab_data.force_action.as_raw(),
            fakexr::ActionState::Float(force),
            hand,
        );
        fakexr::set_action_state(
            grab_data.value_action.as_raw(),
            fakexr::ActionState::Float(value),
            hand,
        );
        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });

        let restrict = match hand {
            LeftHand => left,
            RightHand => right,
        };
        let s = f.get_bool_state_hand(boolact, restrict).unwrap();
        assert_eq!(s.bState, state, "State wrong (line {line})");
        assert_eq!(s.bActive, true, "Active wrong (line {line})");
        assert_eq!(s.bChanged, changed, "Changed wrong (line {line})");
    };

    let grab = super::GrabBindingData::GRAB_THRESHOLD;
    let release = super::GrabBindingData::RELEASE_THRESHOLD;
    value_state_check(grab - 0.1, 1.0, LeftHand, false, false, line!());
    value_state_check(grab - 0.1, 1.0, RightHand, false, false, line!());

    value_state_check(grab, 1.0, LeftHand, true, true, line!());
    value_state_check(grab, 1.0, RightHand, true, true, line!());

    value_state_check(0.0, release, LeftHand, false, true, line!());
    value_state_check(0.0, 1.0, RightHand, true, false, line!());
}

#[test]
fn actions_with_bad_paths() {
    let f = Fixture::new();
    let spaces = f.get_action_handle(c"/actions/set1/in/action with spaces");
    let commas = f.get_action_handle(c"/actions/set1/in/action,with,commas");
    let mixed = f.get_action_handle(c"/actions/set1/in/mixed, action");
    let set1 = f.get_action_set_handle(c"/actions/set1");
    f.load_actions(c"actions_malformed_paths.json");

    fakexr::set_action_state(
        f.get_action::<bool>(spaces),
        fakexr::ActionState::Bool(true),
        LeftHand,
    );
    fakexr::set_action_state(
        f.get_action::<f32>(commas),
        fakexr::ActionState::Float(0.5),
        LeftHand,
    );
    fakexr::set_action_state(
        f.get_action::<bool>(mixed),
        fakexr::ActionState::Bool(true),
        LeftHand,
    );
    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let s = f.get_bool_state(spaces).unwrap();
    assert_eq!(s.bActive, true);
    assert_eq!(s.bState, true);
    assert_eq!(s.bChanged, true);

    let s = f.get_bool_state(mixed).unwrap();
    assert_eq!(s.bActive, true);
    assert_eq!(s.bState, true);
    assert_eq!(s.bChanged, true);

    let mut s = vr::InputAnalogActionData_t::default();
    let ret = f
        .input
        .GetAnalogActionData(commas, &mut s, std::mem::size_of_val(&s) as u32, 0);
    assert_eq!(ret, vr::EVRInputError::None);

    assert_eq!(s.bActive, true);
    assert_eq!(s.x, 0.5);
}

#[test]
fn pose_action_no_restrict() {
    let f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let posel = f.get_action_handle(c"/actions/set1/in/posel");
    let poser = f.get_action_handle(c"/actions/set1/in/poser");

    f.load_actions(c"actions.json");
    f.set_interaction_profile::<ViveWands>(LeftHand);
    f.set_interaction_profile::<ViveWands>(RightHand);
    let session = f.input.openxr.session_data.get().session.as_raw();
    let pose_left = xr::Posef {
        position: xr::Vector3f {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        },
        orientation: xr::Quaternionf::IDENTITY,
    };
    fakexr::set_aim(session, LeftHand, pose_left);

    let pose_right = xr::Posef {
        position: xr::Vector3f {
            x: 0.6,
            y: 0.6,
            z: 0.6,
        },
        orientation: xr::Quaternionf::IDENTITY,
    };
    fakexr::set_aim(session, RightHand, pose_right);

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    for (handle, expected) in [(posel, pose_left), (poser, pose_right)] {
        let actual = f.get_pose(handle, 0).unwrap();
        assert!(actual.bActive);
        let p = actual.pose;
        assert!(p.bPoseIsValid);
        let actual = hmdmatrix34_to_pose(p.mDeviceToAbsoluteTracking);
        compare_pose(expected, actual);
    }
}
