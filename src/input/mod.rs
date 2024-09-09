mod action_manifest;
mod simple_controller;
mod vive_controller;

#[cfg(test)]
mod tests;

use crate::{
    convert::space_relation_to_openvr_pose,
    openxr_data::{self, Hand, OpenXrData, SessionData},
    vr,
};
use action_manifest::InteractionProfile;
use log::{debug, info, trace, warn};
use openxr as xr;
use slotmap::{new_key_type, Key, KeyData, SecondaryMap, SlotMap};
use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, OnceLock, RwLock,
};

new_key_type! {
    struct InputSourceKey;
    struct ActionKey;
    struct ActionSetKey;
}

#[derive(macros::InterfaceImpl)]
#[interface = "IVRInput"]
#[versions(010, 007, 006, 005)]
pub struct Input<C: openxr_data::Compositor> {
    openxr: Arc<OpenXrData<C>>,
    vtables: Vtables<C>,
    input_source_map: RwLock<SlotMap<InputSourceKey, CString>>,
    left_hand_key: InputSourceKey,
    right_hand_key: InputSourceKey,
    action_map: RwLock<SlotMap<ActionKey, Action>>,
    set_map: RwLock<SlotMap<ActionSetKey, String>>,
    loaded_actions_path: OnceLock<PathBuf>,
}

impl<C: openxr_data::Compositor> Input<C> {
    pub fn new(openxr: Arc<OpenXrData<C>>) -> Self {
        let mut map = SlotMap::with_key();
        let left_hand_key = map.insert(c"/user/hand/left".into());
        let right_hand_key = map.insert(c"/user/hand/right".into());
        Self {
            openxr,
            vtables: Default::default(),
            input_source_map: RwLock::new(map),
            action_map: Default::default(),
            set_map: Default::default(),
            loaded_actions_path: OnceLock::new(),
            left_hand_key,
            right_hand_key,
        }
    }

    fn subaction_path_from_handle(&self, handle: vr::VRInputValueHandle_t) -> Option<xr::Path> {
        if handle == vr::k_ulInvalidInputValueHandle {
            Some(xr::Path::NULL)
        } else {
            match InputSourceKey::from(KeyData::from_ffi(handle)) {
                x if x == self.left_hand_key => Some(self.openxr.left_hand.subaction_path),
                x if x == self.right_hand_key => Some(self.openxr.right_hand.subaction_path),
                _ => None,
            }
        }
    }
}

#[derive(Default)]
pub struct InputSessionData {
    loaded_actions: OnceLock<RwLock<LoadedActions>>,
    legacy_actions: OnceLock<LegacyActions>,
}

impl InputSessionData {
    #[inline]
    fn get_loaded_actions(&self) -> Option<std::sync::RwLockReadGuard<'_, LoadedActions>> {
        self.loaded_actions.get().map(|l| l.read().unwrap())
    }
}

enum ActionData {
    Bool(xr::Action<bool>),
    Vector1 {
        action: xr::Action<f32>,
        last_value: AtomicF32,
    },
    Vector2 {
        action: xr::Action<xr::Vector2f>,
        last_value: (AtomicF32, AtomicF32),
    },
    Pose {
        action: xr::Action<xr::Posef>,
        left_space: xr::Space,
        right_space: xr::Space,
    },
    Skeleton {
        action: xr::Action<xr::Posef>,
        space: xr::Space,
        hand: Hand,
    },
    Haptic(xr::Action<xr::Haptic>),
}

macro_rules! get_action_from_handle {
    ($self:expr, $handle:expr, $session_data:ident, $action:ident) => {
        let $session_data = $self.openxr.session_data.get();
        let Some(loaded) = $session_data.input_data.get_loaded_actions() else {
            return vr::EVRInputError::VRInputError_InvalidHandle;
        };

        let $action = match loaded.try_get_action($handle) {
            Ok(action) => action,
            Err(e) => return e,
        };
    };
}

macro_rules! get_subaction_path {
    ($self:expr, $restrict:expr, $data:expr) => {
        match $self.subaction_path_from_handle($restrict) {
            Some(p) => p,
            None => {
                unsafe {
                    $data.write(Default::default());
                }
                return vr::EVRInputError::VRInputError_None;
            }
        }
    };
}

#[derive(Debug)]
struct Action {
    path: String,
}

impl<C: openxr_data::Compositor> vr::IVRInput010_Interface for Input<C> {
    fn GetBindingVariant(
        &self,
        _: vr::VRInputValueHandle_t,
        _: *mut c_char,
        _: u32,
    ) -> vr::EVRInputError {
        crate::warn_unimplemented!("GetBindingVariant");
        vr::EVRInputError::VRInputError_None
    }
    fn OpenBindingUI(
        &self,
        _: *const c_char,
        _: vr::VRActionSetHandle_t,
        _: vr::VRInputValueHandle_t,
        _: bool,
    ) -> vr::EVRInputError {
        todo!()
    }
    fn IsUsingLegacyInput(&self) -> bool {
        todo!()
    }
    fn GetComponentStateForBinding(
        &self,
        _: *const c_char,
        _: *const c_char,
        _: *const vr::InputBindingInfo_t,
        _: u32,
        _: u32,
        _: *mut vr::RenderModel_ComponentState_t,
    ) -> vr::EVRInputError {
        todo!()
    }
    fn ShowBindingsForActionSet(
        &self,
        _: *mut vr::VRActiveActionSet_t,
        _: u32,
        _: u32,
        _: vr::VRInputValueHandle_t,
    ) -> vr::EVRInputError {
        todo!()
    }
    fn ShowActionOrigins(
        &self,
        _: vr::VRActionSetHandle_t,
        _: vr::VRActionHandle_t,
    ) -> vr::EVRInputError {
        todo!()
    }
    fn GetActionBindingInfo(
        &self,
        _: vr::VRActionHandle_t,
        _: *mut vr::InputBindingInfo_t,
        _: u32,
        _: u32,
        returned_binding_info_count: *mut u32,
    ) -> vr::EVRInputError {
        crate::warn_unimplemented!("GetActionBindingInfo");
        if !returned_binding_info_count.is_null() {
            unsafe { *returned_binding_info_count = 0 };
        }
        vr::EVRInputError::VRInputError_None
    }
    fn GetOriginTrackedDeviceInfo(
        &self,
        handle: vr::VRInputValueHandle_t,
        info: *mut vr::InputOriginInfo_t,
        info_size: u32,
    ) -> vr::EVRInputError {
        assert_eq!(
            info_size as usize,
            std::mem::size_of::<vr::InputOriginInfo_t>()
        );

        let key = InputSourceKey::from(KeyData::from_ffi(handle));
        let map = self.input_source_map.read().unwrap();
        if !map.contains_key(key) {
            return vr::EVRInputError::VRInputError_InvalidHandle;
        }

        // Superhot needs this device index to render controllers.
        let index = match key {
            x if x == self.left_hand_key => Hand::Left as u32,
            x if x == self.right_hand_key => Hand::Right as u32,
            _ => {
                unsafe {
                    info.write(Default::default());
                }
                return vr::EVRInputError::VRInputError_None;
            }
        };

        unsafe {
            *info.as_mut().unwrap() = vr::InputOriginInfo_t {
                devicePath: handle,
                trackedDeviceIndex: index,
                rchRenderModelComponentName: [0; 128],
            };
        }
        vr::EVRInputError::VRInputError_None
    }
    fn GetOriginLocalizedName(
        &self,
        _: vr::VRInputValueHandle_t,
        _: *mut c_char,
        _: u32,
        _: i32,
    ) -> vr::EVRInputError {
        crate::warn_unimplemented!("GetOriginLocalizedName");
        vr::EVRInputError::VRInputError_None
    }
    fn GetActionOrigins(
        &self,
        _: vr::VRActionSetHandle_t,
        _: vr::VRActionHandle_t,
        _: *mut vr::VRInputValueHandle_t,
        _: u32,
    ) -> vr::EVRInputError {
        crate::warn_unimplemented!("GetActionOrigins");
        vr::EVRInputError::VRInputError_None
    }
    fn TriggerHapticVibrationAction(
        &self,
        action: vr::VRActionHandle_t,
        start_seconds_from_now: f32,
        duration_seconds: f32,
        frequency: f32,
        amplitude: f32,
        restrict_to_device: vr::VRInputValueHandle_t,
    ) -> vr::EVRInputError {
        get_action_from_handle!(self, action, session_data, action);
        let Some(subaction_path) = self.subaction_path_from_handle(restrict_to_device) else {
            return vr::EVRInputError::VRInputError_None;
        };

        let ActionData::Haptic(action) = action else {
            return vr::EVRInputError::VRInputError_WrongType;
        };

        if start_seconds_from_now > 0.0 {
            warn!("start_seconds_from_now: {start_seconds_from_now}")
        }

        action
            .apply_feedback(
                &session_data.session,
                subaction_path,
                &xr::HapticVibration::new()
                    .amplitude(amplitude.clamp(0.0, 1.0))
                    .frequency(frequency)
                    .duration(xr::Duration::from_nanos((duration_seconds * 1e9) as _)),
            )
            .unwrap();

        vr::EVRInputError::VRInputError_None
    }
    fn DecompressSkeletalBoneData(
        &self,
        _: *const std::os::raw::c_void,
        _: u32,
        _: vr::EVRSkeletalTransformSpace,
        _: *mut vr::VRBoneTransform_t,
        _: u32,
    ) -> vr::EVRInputError {
        todo!()
    }
    fn GetSkeletalBoneDataCompressed(
        &self,
        _: vr::VRActionHandle_t,
        _: vr::EVRSkeletalMotionRange,
        _: *mut std::os::raw::c_void,
        _: u32,
        _: *mut u32,
    ) -> vr::EVRInputError {
        todo!()
    }
    fn GetSkeletalSummaryData(
        &self,
        action: vr::VRActionHandle_t,
        _: vr::EVRSummaryType,
        data: *mut vr::VRSkeletalSummaryData_t,
    ) -> vr::EVRInputError {
        get_action_from_handle!(self, action, session_data, _action);
        unsafe {
            data.write(vr::VRSkeletalSummaryData_t {
                flFingerSplay: [0.2; 4],
                flFingerCurl: [0.0; 5],
            })
        }
        vr::EVRInputError::VRInputError_None
    }
    fn GetSkeletalBoneData(
        &self,
        _: vr::VRActionHandle_t,
        _: vr::EVRSkeletalTransformSpace,
        _: vr::EVRSkeletalMotionRange,
        _: *mut vr::VRBoneTransform_t,
        _: u32,
    ) -> vr::EVRInputError {
        crate::warn_unimplemented!("GetSkeletalBoneData");
        vr::EVRInputError::VRInputError_None
    }
    fn GetSkeletalTrackingLevel(
        &self,
        _: vr::VRActionHandle_t,
        level: *mut vr::EVRSkeletalTrackingLevel,
    ) -> vr::EVRInputError {
        unsafe {
            *level = vr::EVRSkeletalTrackingLevel::VRSkeletalTracking_Estimated;
        }
        vr::EVRInputError::VRInputError_None
    }
    fn GetSkeletalReferenceTransforms(
        &self,
        _: vr::VRActionHandle_t,
        _: vr::EVRSkeletalTransformSpace,
        _: vr::EVRSkeletalReferencePose,
        _: *mut vr::VRBoneTransform_t,
        _: u32,
    ) -> vr::EVRInputError {
        crate::warn_unimplemented!("GetSkeletalReferenceTransforms");
        vr::EVRInputError::VRInputError_None
    }
    fn GetBoneName(
        &self,
        _: vr::VRActionHandle_t,
        _: vr::BoneIndex_t,
        _: *mut c_char,
        _: u32,
    ) -> vr::EVRInputError {
        todo!()
    }
    fn GetBoneHierarchy(
        &self,
        _: vr::VRActionHandle_t,
        _: *mut vr::BoneIndex_t,
        _: u32,
    ) -> vr::EVRInputError {
        todo!()
    }
    fn GetBoneCount(&self, _: vr::VRActionHandle_t, _: *mut u32) -> vr::EVRInputError {
        crate::warn_unimplemented!("GetBoneCount");
        vr::EVRInputError::VRInputError_None
    }
    fn SetDominantHand(&self, _: vr::ETrackedControllerRole) -> vr::EVRInputError {
        todo!()
    }
    fn GetDominantHand(&self, _: *mut vr::ETrackedControllerRole) -> vr::EVRInputError {
        crate::warn_unimplemented!("GetDominantHand");
        vr::EVRInputError::VRInputError_None
    }
    fn GetSkeletalActionData(
        &self,
        action: vr::VRActionHandle_t,
        action_data: *mut vr::InputSkeletalActionData_t,
        _action_data_size: u32,
    ) -> vr::EVRInputError {
        //assert_eq!(
        //    action_data_size as usize,
        //    std::mem::size_of::<vr::InputSkeletalActionData_t>()
        //);

        let data = self.openxr.session_data.get();
        let Some(loaded) = data.input_data.get_loaded_actions() else {
            return vr::EVRInputError::VRInputError_InvalidHandle;
        };
        let (action, origin) = match loaded.try_get_action(action) {
            Ok(ActionData::Skeleton { action, hand, .. }) => (
                action,
                match hand {
                    Hand::Left => self.left_hand_key.data().as_ffi(),
                    Hand::Right => self.right_hand_key.data().as_ffi(),
                },
            ),
            Ok(_) => return vr::EVRInputError::VRInputError_WrongType,
            Err(e) => return e,
        };

        unsafe {
            std::ptr::addr_of_mut!((*action_data).bActive)
                .write(action.is_active(&data.session, xr::Path::NULL).unwrap());
            std::ptr::addr_of_mut!((*action_data).activeOrigin).write(origin);
        }
        vr::EVRInputError::VRInputError_None
    }
    fn GetPoseActionDataForNextFrame(
        &self,
        action: vr::VRActionHandle_t,
        origin: vr::ETrackingUniverseOrigin,
        action_data: *mut vr::InputPoseActionData_t,
        action_data_size: u32,
        restrict_to_device: vr::VRInputValueHandle_t,
    ) -> vr::EVRInputError {
        assert_eq!(
            action_data_size as usize,
            std::mem::size_of::<vr::InputPoseActionData_t>()
        );

        let map = self.action_map.read().unwrap();
        let key = ActionKey::from(KeyData::from_ffi(action));
        trace!("getting pose for {}", map[key].path);

        let data = self.openxr.session_data.get();
        let Some(loaded) = data.input_data.get_loaded_actions() else {
            return vr::EVRInputError::VRInputError_InvalidHandle;
        };

        let subaction_path = match self.subaction_path_from_handle(restrict_to_device) {
            Some(p) => p,
            None => {
                unsafe {
                    action_data.write(Default::default());
                }
                return vr::EVRInputError::VRInputError_None;
            }
        };

        let (action, space, active_origin) = match loaded.try_get_action(action) {
            Ok(ActionData::Pose {
                action,
                left_space,
                right_space,
            }) => match subaction_path {
                x if x == self.openxr.left_hand.subaction_path => {
                    (action, left_space, self.left_hand_key.data().as_ffi())
                }
                x if x == self.openxr.right_hand.subaction_path => {
                    (action, right_space, self.right_hand_key.data().as_ffi())
                }
                _ => unreachable!(),
            },
            Ok(ActionData::Skeleton {
                action,
                space,
                hand,
            }) => (
                action,
                space,
                match hand {
                    Hand::Left => self.left_hand_key.data().as_ffi(),
                    Hand::Right => self.right_hand_key.data().as_ffi(),
                },
            ),
            Ok(_) => return vr::EVRInputError::VRInputError_WrongType,
            Err(e) => return e,
        };

        let active = action.is_active(&data.session, subaction_path).unwrap();
        let base = data.get_space_for_origin(origin);
        let (loc, velo) = space.relate(base, self.openxr.display_time.get()).unwrap();
        unsafe {
            action_data.write(vr::InputPoseActionData_t {
                bActive: active,
                activeOrigin: active_origin,
                pose: space_relation_to_openvr_pose(loc, velo),
            });

            trace!("pose: {:#?}", (*action_data).pose.mDeviceToAbsoluteTracking)
        }

        vr::EVRInputError::VRInputError_None
    }

    fn GetPoseActionDataRelativeToNow(
        &self,
        action: vr::VRActionHandle_t,
        origin: vr::ETrackingUniverseOrigin,
        _seconds_from_now: f32,
        action_data: *mut vr::InputPoseActionData_t,
        action_data_size: u32,
        restrict_to_device: vr::VRInputValueHandle_t,
    ) -> vr::EVRInputError {
        self.GetPoseActionDataForNextFrame(
            action,
            origin,
            action_data,
            action_data_size,
            restrict_to_device,
        )
    }

    fn GetAnalogActionData(
        &self,
        handle: vr::VRActionHandle_t,
        action_data: *mut vr::InputAnalogActionData_t,
        action_data_size: u32,
        restrict_to_device: vr::VRInputValueHandle_t,
    ) -> vr::EVRInputError {
        assert_eq!(
            action_data_size as usize,
            std::mem::size_of::<vr::InputAnalogActionData_t>()
        );

        get_action_from_handle!(self, handle, session_data, action);
        let subaction_path = get_subaction_path!(self, restrict_to_device, action_data);

        let (state, delta) = match action {
            ActionData::Vector1 { action, last_value } => {
                let state = action.state(&session_data.session, subaction_path).unwrap();
                let delta = xr::Vector2f {
                    x: state.current_state - last_value.load(),
                    y: 0.0,
                };
                last_value.store(state.current_state);
                (
                    xr::ActionState::<xr::Vector2f> {
                        current_state: xr::Vector2f {
                            x: state.current_state,
                            y: 0.0,
                        },
                        changed_since_last_sync: state.changed_since_last_sync,
                        last_change_time: state.last_change_time,
                        is_active: state.is_active,
                    },
                    delta,
                )
            }
            ActionData::Vector2 { action, last_value } => {
                let state = action.state(&session_data.session, subaction_path).unwrap();
                let delta = xr::Vector2f {
                    x: state.current_state.x - last_value.0.load(),
                    y: state.current_state.y - last_value.1.load(),
                };
                last_value.0.store(state.current_state.x);
                last_value.1.store(state.current_state.y);
                (state, delta)
            }
            _ => return vr::EVRInputError::VRInputError_WrongType,
        };
        unsafe {
            action_data.write(vr::InputAnalogActionData_t {
                bActive: state.is_active,
                activeOrigin: 0,
                x: state.current_state.x,
                deltaX: delta.x,
                y: state.current_state.y,
                deltaY: delta.y,
                ..Default::default()
            });
        }

        vr::EVRInputError::VRInputError_None
    }

    fn GetDigitalActionData(
        &self,
        action: vr::VRActionHandle_t,
        action_data: *mut vr::InputDigitalActionData_t,
        action_data_size: u32,
        restrict_to_device: vr::VRInputValueHandle_t,
    ) -> vr::EVRInputError {
        assert_eq!(
            action_data_size as usize,
            std::mem::size_of::<vr::InputDigitalActionData_t>()
        );

        let data = self.openxr.session_data.get();
        let Some(loaded) = data.input_data.get_loaded_actions() else {
            return vr::EVRInputError::VRInputError_InvalidHandle;
        };

        let subaction_path = match self.subaction_path_from_handle(restrict_to_device) {
            Some(p) => p,
            None => {
                unsafe {
                    action_data.write(Default::default());
                }
                return vr::EVRInputError::VRInputError_None;
            }
        };

        let action = match loaded.try_get_action(action) {
            Ok(action) => {
                let ActionData::Bool(action) = &action else {
                    return vr::EVRInputError::VRInputError_WrongType;
                };
                action
            }
            Err(e) => return e,
        };

        let state = action.state(&data.session, subaction_path).unwrap();
        unsafe {
            action_data.write(vr::InputDigitalActionData_t {
                bActive: state.is_active,
                bState: state.current_state,
                activeOrigin: 0, // TODO
                bChanged: state.changed_since_last_sync,
                fUpdateTime: 0.0, // TODO
            });
        }

        vr::EVRInputError::VRInputError_None
    }

    fn UpdateActionState(
        &self,
        active_sets: *mut vr::VRActiveActionSet_t,
        active_set_size: u32,
        active_set_count: u32,
    ) -> vr::EVRInputError {
        assert_eq!(
            active_set_size as usize,
            std::mem::size_of::<vr::VRActiveActionSet_t>()
        );
        // alyx
        if active_set_count == 0 {
            return vr::EVRInputError::VRInputError_NoActiveActionSet;
        }

        let active_sets =
            unsafe { std::slice::from_raw_parts(active_sets, active_set_count as usize) };

        if active_sets
            .iter()
            .any(|set| set.ulRestrictedToDevice != vr::k_ulInvalidInputValueHandle)
        {
            crate::warn_once!("Per device action set restriction is not implemented yet.");
        }

        let data = self.openxr.session_data.get();
        let Some(actions) = data.input_data.get_loaded_actions() else {
            return vr::EVRInputError::VRInputError_InvalidParam;
        };

        let mut sync_sets = Vec::with_capacity(active_sets.len() + 1);
        for set in active_sets {
            let key = ActionSetKey::from(KeyData::from_ffi(set.ulActionSet));
            let m = self.set_map.read().unwrap();
            let name = m.get(key);
            let Some(set) = actions.sets.get(key) else {
                debug!("Application passed invalid action set key: {key:?} ({name:?})");
                return vr::EVRInputError::VRInputError_InvalidHandle;
            };
            debug!("Activating set {}", name.unwrap());
            sync_sets.push(set.into());
        }

        let legacy = data.input_data.legacy_actions.get().unwrap();
        sync_sets.push(xr::ActiveActionSet::new(&legacy.set));
        legacy.packet_num.fetch_add(1, Ordering::Relaxed);

        data.session.sync_actions(&sync_sets).unwrap();

        vr::EVRInputError::VRInputError_None
    }

    fn GetInputSourceHandle(
        &self,
        input_source_path: *const c_char,
        handle: *mut vr::VRInputValueHandle_t,
    ) -> vr::EVRInputError {
        let path = unsafe { CStr::from_ptr(input_source_path) };

        let ret = {
            let guard = self.input_source_map.read().unwrap();
            match guard.iter().find(|(_, src)| src.as_c_str() == path) {
                Some((key, _)) => key.data().as_ffi(),
                None => {
                    drop(guard);
                    let mut guard = self.input_source_map.write().unwrap();
                    let key = guard.insert(path.into());
                    key.data().as_ffi()
                }
            }
        };
        if let Some(handle) = unsafe { handle.as_mut() } {
            debug!("requested handle for path {path:?}: {ret}");
            *handle = ret;
            vr::EVRInputError::VRInputError_None
        } else {
            vr::EVRInputError::VRInputError_InvalidParam
        }
    }

    fn GetActionHandle(
        &self,
        action_name: *const c_char,
        handle: *mut vr::VRActionHandle_t,
    ) -> vr::EVRInputError {
        let name = unsafe { CStr::from_ptr(action_name) }
            .to_string_lossy()
            .to_lowercase();
        let guard = self.action_map.read().unwrap();
        let val = match guard.iter().find(|(_, action)| action.path == name) {
            Some((key, _)) => key.data().as_ffi(),
            None => {
                drop(guard);
                let mut guard = self.action_map.write().unwrap();
                let key = guard.insert(Action { path: name.into() });
                key.data().as_ffi()
            }
        };

        if let Some(handle) = unsafe { handle.as_mut() } {
            *handle = val;
            vr::EVRInputError::VRInputError_None
        } else {
            vr::EVRInputError::VRInputError_InvalidParam
        }
    }

    fn GetActionSetHandle(
        &self,
        action_set_name: *const c_char,
        handle: *mut vr::VRActionSetHandle_t,
    ) -> vr::EVRInputError {
        let name = unsafe { CStr::from_ptr(action_set_name) }
            .to_string_lossy()
            .to_lowercase();
        let guard = self.set_map.read().unwrap();
        let val = match guard.iter().find(|(_, set)| **set == name) {
            Some((key, _)) => key.data().as_ffi(),
            None => {
                drop(guard);
                let mut guard = self.set_map.write().unwrap();
                let key = guard.insert(name.into());
                key.data().as_ffi()
            }
        };

        if let Some(handle) = unsafe { handle.as_mut() } {
            *handle = val;
            vr::EVRInputError::VRInputError_None
        } else {
            vr::EVRInputError::VRInputError_InvalidParam
        }
    }

    fn SetActionManifestPath(&self, path: *const c_char) -> vr::EVRInputError {
        let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
        let path = std::path::Path::new(&*path);
        info!("loading action manifest from {path:?}");

        // We need to restart the session if the legacy actions have already been attached.
        let mut data = self.openxr.session_data.get();
        if data.input_data.legacy_actions.get().is_some() {
            drop(data);
            self.openxr.restart_session();
            data = self.openxr.session_data.get();
        }
        match self.load_action_manifest(&*data, path) {
            Ok(_) => vr::EVRInputError::VRInputError_None,
            Err(e) => e,
        }
    }
}

impl<C: openxr_data::Compositor> vr::IVRInput005On006 for Input<C> {
    #[inline]
    fn GetSkeletalSummaryData(
        &self,
        action: vr::VRActionHandle_t,
        summary_data: *mut vr::VRSkeletalSummaryData_t,
    ) -> vr::EVRInputError {
        <Self as vr::IVRInput010_Interface>::GetSkeletalSummaryData(
            self,
            action,
            vr::EVRSummaryType::VRSummaryType_FromAnimation,
            summary_data,
        )
    }

    #[inline]
    fn GetPoseActionData(
        &self,
        action: vr::VRActionHandle_t,
        origin: vr::ETrackingUniverseOrigin,
        seconds_from_now: f32,
        action_data: *mut vr::InputPoseActionData_t,
        action_data_size: u32,
        restrict_to_device: vr::VRInputValueHandle_t,
    ) -> vr::EVRInputError {
        <Self as vr::IVRInput010_Interface>::GetPoseActionDataRelativeToNow(
            self,
            action,
            origin,
            seconds_from_now,
            action_data,
            action_data_size,
            restrict_to_device,
        )
    }
}

impl<C: openxr_data::Compositor> Input<C> {
    pub fn get_poses(
        &self,
        poses: &mut [vr::TrackedDevicePose_t],
        origin: Option<vr::ETrackingUniverseOrigin>,
    ) {
        poses[0] = self.get_hmd_pose(origin);

        if poses.len() > Hand::Left as usize {
            poses[Hand::Left as usize] = self
                .get_controller_pose(Hand::Left, origin)
                .unwrap_or_default();
        }
        if poses.len() > Hand::Right as usize {
            poses[Hand::Right as usize] = self
                .get_controller_pose(Hand::Right, origin)
                .unwrap_or_default();
        }
    }

    fn get_hmd_pose(&self, origin: Option<vr::ETrackingUniverseOrigin>) -> vr::TrackedDevicePose_t {
        let data = self.openxr.session_data.get();
        let (hmd_location, hmd_velocity) = {
            data.view_space
                .relate(
                    data.get_space_for_origin(origin.unwrap_or(data.current_origin)),
                    self.openxr.display_time.get(),
                )
                .unwrap()
        };

        space_relation_to_openvr_pose(hmd_location, hmd_velocity)
    }

    pub fn get_controller_pose(
        &self,
        hand: Hand,
        origin: Option<vr::ETrackingUniverseOrigin>,
    ) -> Option<vr::TrackedDevicePose_t> {
        let data = self.openxr.session_data.get();
        // bail out if legacy actions haven't been set up
        let actions = data.input_data.legacy_actions.get()?;
        let space = match hand {
            Hand::Left => &actions.left_space,
            Hand::Right => &actions.right_space,
        };

        let (loc, velo) = space
            .relate(
                data.get_space_for_origin(origin.unwrap_or(data.current_origin)),
                self.openxr.display_time.get(),
            )
            .unwrap();

        Some(space_relation_to_openvr_pose(loc, velo))
    }

    pub fn get_legacy_controller_state(
        &self,
        device_index: vr::TrackedDeviceIndex_t,
        state: *mut vr::VRControllerState_t,
        state_size: u32,
    ) -> bool {
        if state_size as usize != std::mem::size_of::<vr::VRControllerState_t>() {
            warn!(
                "Got an unexpected size for VRControllerState_t (expected {}, got {state_size})",
                std::mem::size_of::<vr::VRControllerState_t>()
            );
            return false;
        }

        let data = self.openxr.session_data.get();
        let Some(actions) = data.input_data.legacy_actions.get() else {
            debug!("tried getting controller state, but legacy actions aren't ready");
            return false;
        };

        let Ok(hand) = Hand::try_from(device_index) else {
            debug!("requested controller state for invalid device index: {device_index}");
            return false;
        };

        let hand_path = match hand {
            Hand::Left => self.openxr.left_hand.subaction_path,
            Hand::Right => self.openxr.right_hand.subaction_path,
        };

        let data = self.openxr.session_data.get();

        // Adapted from openvr.h
        fn button_mask_from_id(id: vr::EVRButtonId) -> u64 {
            1_u64 << (id as u32)
        }

        let state = unsafe { state.as_mut() }.unwrap();
        *state = Default::default();

        state.unPacketNum = actions.packet_num.load(Ordering::Relaxed);

        let mut read_button = |id, action: &xr::Action<bool>| {
            let val = action
                .state(&data.session, hand_path)
                .unwrap()
                .current_state as u64
                * u64::MAX;
            state.ulButtonPressed |= button_mask_from_id(id) & val;
        };

        read_button(
            vr::EVRButtonId::k_EButton_SteamVR_Trigger,
            &actions.trigger_click,
        );
        read_button(
            vr::EVRButtonId::k_EButton_ApplicationMenu,
            &actions.app_menu,
        );

        let t = actions.trigger.state(&data.session, hand_path).unwrap();
        state.rAxis[1] = vr::VRControllerAxis_t {
            x: t.current_state,
            y: 0.0,
        };

        true
    }

    pub fn frame_start_update(&self) {
        let data = self.openxr.session_data.get();
        // If the game has loaded actions, we don't need to sync the state because the game should
        // be doing it itself (with UpdateActionState)
        if data.input_data.loaded_actions.get().is_some() {
            return;
        }

        match data.input_data.legacy_actions.get() {
            Some(actions) => {
                data.session
                    .sync_actions(&[xr::ActiveActionSet::new(&actions.set)])
                    .unwrap();

                actions.packet_num.fetch_add(1, Ordering::Relaxed);
            }
            None => {
                // If we haven't created our legacy actions yet but we're getting our per frame
                // update, go ahead and create them
                // This will force us to have to restart the session when we get an action
                // manifest, but that's fine, because in the event an action manifest is never
                // loaded (legacy input), having the legacy actions loaded and synced enables us to
                // determine if controllers are actually connected, and some games avoid getting
                // controller state unless they are reported as actually connected.

                // Make sure we're using the real session already
                // This avoids a scenario where we could go:
                // 1. attach legacy inputs
                // 2. restart session to attach action manifest
                // 3. restart to use real session
                if !data.is_real_session() {
                    debug!(
                        "Couldn't set up legacy actions because we're not in the real session yet."
                    );
                    return;
                }
                let legacy = LegacyActions::new(
                    &self.openxr.instance,
                    &data.session,
                    self.openxr.left_hand.subaction_path,
                    self.openxr.right_hand.subaction_path,
                );
                setup_legacy_bindings(&self.openxr.instance, &data.session, &legacy);
                data.input_data
                    .legacy_actions
                    .set(legacy)
                    .unwrap_or_else(|_| unreachable!());
            }
        }
    }

    pub fn get_controller_string_tracked_property(
        &self,
        hand: Hand,
        property: vr::ETrackedDeviceProperty,
    ) -> Option<&'static CStr> {
        use vr::ETrackedDeviceProperty::*;

        match property {
            Prop_ControllerType_String => {
                static PATH_MAP: OnceLock<Vec<(xr::Path, &'static CStr)>> = OnceLock::new();
                let map = PATH_MAP.get_or_init(|| {
                    let instance = &self.openxr.instance;
                    let mut v = Vec::new();
                    let out = &mut v;
                    action_manifest::for_each_profile! {<'a>(
                        instance: &'a xr::Instance,
                        out: &'a mut Vec<(xr::Path, &'static CStr)>
                    ) {
                        out.push((
                            instance.string_to_path(P::PROFILE_PATH).unwrap(),
                            P::OPENVR_CONTROLLER_TYPE,
                        ));
                    }}
                    v
                });

                let hand = match hand {
                    Hand::Left => &self.openxr.left_hand,
                    Hand::Right => &self.openxr.right_hand,
                };
                let profile = hand.interaction_profile.load();

                let controller_type = map
                    .iter()
                    .find_map(|(path, ty)| (*path == profile).then_some(*ty));

                controller_type
            }
            _ => None,
        }
    }

    pub fn post_session_restart(&self, data: &SessionData) {
        // This function is called while a write lock is called on the session, and as such should
        // not use self.openxr.session_data.get().
        if let Some(path) = self.loaded_actions_path.get() {
            self.load_action_manifest(data, path).unwrap();
        }
    }
}

fn setup_legacy_bindings(
    instance: &xr::Instance,
    session: &xr::Session<xr::vulkan::Vulkan>,
    actions: &LegacyActions,
) {
    debug!("setting up legacy bindings");

    action_manifest::for_each_profile! {<'a>(
        instance: &'a xr::Instance,
        actions: &'a LegacyActions
    ) {
        let stp = |s| instance.string_to_path(s).unwrap();
        let bindings = P::legacy_bindings(stp, actions);
        let profile = stp(P::PROFILE_PATH);
        instance
            .suggest_interaction_profile_bindings(profile, &bindings)
            .unwrap();
    }}

    session.attach_action_sets(&[&actions.set]).unwrap();
    session
        .sync_actions(&[xr::ActiveActionSet::new(&actions.set)])
        .unwrap();
}

struct LoadedActions {
    sets: SecondaryMap<ActionSetKey, xr::ActionSet>,
    actions: SecondaryMap<ActionKey, ActionData>,
}

impl LoadedActions {
    fn try_get_action(
        &self,
        handle: vr::VRActionHandle_t,
    ) -> Result<&ActionData, vr::EVRInputError> {
        let key = ActionKey::from(KeyData::from_ffi(handle));
        self.actions
            .get(key)
            .ok_or(vr::EVRInputError::VRInputError_InvalidHandle)
    }
}

struct LegacyActions {
    set: xr::ActionSet,
    pose: xr::Action<xr::Posef>,
    app_menu: xr::Action<bool>,
    trigger_click: xr::Action<bool>,
    trigger: xr::Action<f32>,
    packet_num: AtomicU32,
    left_space: xr::Space,
    right_space: xr::Space,
}

impl LegacyActions {
    fn new<'a>(
        instance: &'a xr::Instance,
        session: &'a xr::Session<xr::vulkan::Vulkan>,
        left_hand: xr::Path,
        right_hand: xr::Path,
    ) -> Self {
        debug!("creating legacy actions");
        let leftright = [left_hand, right_hand];
        let set = instance
            .create_action_set("xrizer-legacy-set", "XRizer Legacy Set", 0)
            .unwrap();
        let pose = set.create_action("pose", "Pose", &leftright).unwrap();
        let trigger_click = set
            .create_action("trigger-click", "Trigger Click", &leftright)
            .unwrap();
        let trigger = set.create_action("trigger", "Trigger", &leftright).unwrap();
        let app_menu = set
            .create_action("app-menu", "Application Menu", &leftright)
            .unwrap();

        let left_space = pose
            .create_space(session, left_hand, xr::Posef::IDENTITY)
            .unwrap();
        let right_space = pose
            .create_space(session, right_hand, xr::Posef::IDENTITY)
            .unwrap();

        Self {
            set,
            pose,
            app_menu,
            trigger_click,
            trigger,
            packet_num: 0.into(),
            left_space,
            right_space,
        }
    }
}

struct AtomicF32(AtomicU32);
impl AtomicF32 {
    fn new(value: f32) -> Self {
        Self(value.to_bits().into())
    }

    fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    fn store(&self, value: f32) {
        self.0.store(value.to_bits(), Ordering::Relaxed)
    }
}
