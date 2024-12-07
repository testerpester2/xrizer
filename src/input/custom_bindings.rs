use openxr as xr;
use std::f32::consts::{FRAC_PI_4, PI};
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct BoolActionData {
    pub action: xr::Action<bool>,
    pub dpad_data: Option<DpadData>,
    pub grab_data: Option<GrabBindingData>,
    pub toggle_data: Option<ToggleData>,
}

impl BoolActionData {
    pub fn new(action: xr::Action<bool>) -> Self {
        Self {
            action,
            dpad_data: None,
            grab_data: None,
            toggle_data: None,
        }
    }

    pub fn state<G>(
        &self,
        session: &xr::Session<G>,
        subaction_path: xr::Path,
    ) -> xr::Result<xr::ActionState<bool>> {
        // First, we try the normal boolean action
        // We may have dpad data, but some controller types may not have been bound to dpad inputs,
        // so we need to try the regular action first.
        let mut state = self.action.state(session, subaction_path)?;

        if state.is_active && state.current_state {
            return Ok(state);
        }

        // state.is_active being false implies there's nothing bound to the action, so then we try
        // our dpad input, if available.

        if let Some(data) = &self.dpad_data {
            if let Some(s) = data.state(session)? {
                state = s;
                if s.current_state {
                    return Ok(s);
                }
            }
        }

        if let Some(data) = &self.grab_data {
            if let Some(state) = data.grabbed(session, subaction_path)? {
                return Ok(state);
            }
        }

        if let Some(data) = &self.toggle_data {
            if let Some(s) = data.state(session, subaction_path)? {
                state = s;
                if s.current_state {
                    return Ok(s);
                }
            }
        }

        Ok(state)
    }
}

pub(super) struct FloatActionData {
    pub action: xr::Action<f32>,
    pub last_value: super::AtomicF32,
    pub grab_data: Option<GrabBindingData>,
}

impl FloatActionData {
    pub fn new(action: xr::Action<f32>) -> Self {
        Self {
            action,
            last_value: Default::default(),
            grab_data: None,
        }
    }

    pub fn state<G>(
        &self,
        session: &xr::Session<G>,
        subaction_path: xr::Path,
    ) -> xr::Result<xr::ActionState<f32>> {
        let state = self.action.state(session, subaction_path)?;
        if state.is_active {
            return Ok(state);
        }

        if let Some(data) = &self.grab_data {
            if data.grabbed(session, subaction_path)?.is_some() {
                todo!("handle grab bindings for float actions");
            }
        }

        Ok(state)
    }
}
#[derive(Debug)]
pub(super) enum DpadDirection {
    North,
    East,
    South,
    West,
    Center,
}

pub(super) struct DpadData {
    pub parent: xr::Action<xr::Vector2f>,
    pub click_or_touch: Option<xr::Action<bool>>,
    pub direction: DpadDirection,
    pub last_state: AtomicBool,
}

impl DpadData {
    const CENTER_ZONE: f32 = 0.5;
    pub fn state<G>(&self, session: &xr::Session<G>) -> xr::Result<Option<xr::ActionState<bool>>> {
        let parent_state = self.parent.state(session, xr::Path::NULL)?;
        let mut ret_state = xr::ActionState {
            current_state: false,
            last_change_time: parent_state.last_change_time, // TODO: this is wrong
            changed_since_last_sync: false,
            is_active: parent_state.is_active,
        };

        let active = self
            .click_or_touch
            .as_ref()
            .map(|a| {
                // If this action isn't bound in the current interaction profile,
                // is_active will be false - in this case, it's probably a joystick touch dpad, in
                // which case we still want to read the current state.
                a.state(session, xr::Path::NULL)
                    .map(|s| !s.is_active || s.current_state)
            })
            .unwrap_or(Ok(true))?;

        if !active {
            return Ok(None);
        }

        // convert to polar coordinates
        let xr::Vector2f { x, y } = parent_state.current_state;
        let radius = x.hypot(y);
        let angle = y.atan2(x);

        // pi/2 wedges, no overlap
        let in_bounds = match self.direction {
            DpadDirection::North => {
                radius >= Self::CENTER_ZONE && (FRAC_PI_4..=3.0 * FRAC_PI_4).contains(&angle)
            }
            DpadDirection::East => {
                radius >= Self::CENTER_ZONE && (-FRAC_PI_4..=FRAC_PI_4).contains(&angle)
            }
            DpadDirection::South => {
                radius >= Self::CENTER_ZONE && (-3.0 * FRAC_PI_4..=-FRAC_PI_4).contains(&angle)
            }
            // west section is disjoint with atan2
            DpadDirection::West => {
                radius >= Self::CENTER_ZONE
                    && ((3.0 * FRAC_PI_4..=PI).contains(&angle)
                        || (-PI..=-3.0 * FRAC_PI_4).contains(&angle))
            }
            DpadDirection::Center => radius < Self::CENTER_ZONE,
        };

        ret_state.current_state = in_bounds;
        if self
            .last_state
            .compare_exchange(!in_bounds, in_bounds, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            ret_state.changed_since_last_sync = true;
        }

        Ok(Some(ret_state))
    }
}

pub(super) struct GrabBindingData {
    pub force_action: xr::Action<f32>,
    pub value_action: xr::Action<f32>,
    pub last_state: [(xr::Path, AtomicBool); 2],
}

impl GrabBindingData {
    pub fn new(force: xr::Action<f32>, value: xr::Action<f32>, paths: [xr::Path; 2]) -> Self {
        assert!(paths.iter().copied().all(|p| p != xr::Path::NULL));
        Self {
            force_action: force,
            value_action: value,
            last_state: paths.map(|p| (p, false.into())),
        }
    }

    // These values were determined empirically.

    /// How much force to apply to begin a grab
    pub const GRAB_THRESHOLD: f32 = 0.10;
    /// How much the value component needs to be to release the grab.
    pub const RELEASE_THRESHOLD: f32 = 0.35;

    /// Returns None if the grab data is not active.
    pub fn grabbed<G>(
        &self,
        session: &xr::Session<G>,
        subaction_path: xr::Path,
    ) -> xr::Result<Option<xr::ActionState<bool>>> {
        // FIXME: the way this function calculates changed_since_last_sync is incorrect, as it will
        // always be false if this is called more than once between syncs. What should be done is
        // the state should be updated in UpdateActionState, but that may have other implications
        // I currently don't feel like thinking about, as this works and I haven't seen games grab action
        // state more than once beteween syncs.
        let force_state = self.force_action.state(session, subaction_path)?;
        let value_state = self.value_action.state(session, subaction_path)?;
        if !force_state.is_active || !value_state.is_active {
            Ok(None)
        } else {
            let (grabbed, changed_since_last_sync) = match &self.last_state {
                [(path, old_state), _] | [_, (path, old_state)] if *path == subaction_path => {
                    let s = old_state.load(Ordering::Relaxed);
                    let grabbed = (!s && force_state.current_state >= Self::GRAB_THRESHOLD)
                        || (s && value_state.current_state > Self::RELEASE_THRESHOLD);
                    let changed = old_state
                        .compare_exchange(!grabbed, grabbed, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok();
                    (grabbed, changed)
                }
                [(_, old_state1), (_, old_state2)] if subaction_path == xr::Path::NULL => {
                    let s =
                        old_state1.load(Ordering::Relaxed) || old_state2.load(Ordering::Relaxed);
                    let grabbed = (!s && force_state.current_state >= Self::GRAB_THRESHOLD)
                        || (s && value_state.current_state > Self::RELEASE_THRESHOLD);
                    let cmpex = |state: &AtomicBool| {
                        state
                            .compare_exchange(
                                !grabbed,
                                grabbed,
                                Ordering::Relaxed,
                                Ordering::Relaxed,
                            )
                            .is_ok()
                    };
                    let changed1 = cmpex(old_state1);
                    let changed2 = cmpex(old_state2);
                    (grabbed, changed1 || changed2)
                }
                _ => unreachable!(),
            };

            Ok(Some(xr::ActionState {
                current_state: grabbed,
                changed_since_last_sync,
                last_change_time: force_state.last_change_time,
                is_active: true,
            }))
        }
    }
}

pub(super) struct ToggleData {
    pub action: xr::Action<bool>,
    pub last_state: [(xr::Path, AtomicBool); 2],
}

impl ToggleData {
    pub fn new(action: xr::Action<bool>, paths: [xr::Path; 2]) -> Self {
        Self {
            action,
            last_state: paths.map(|p| (p, false.into())),
        }
    }

    pub fn state<G>(
        &self,
        session: &xr::Session<G>,
        subaction_path: xr::Path,
    ) -> xr::Result<Option<xr::ActionState<bool>>> {
        let state = self.action.state(session, subaction_path)?;
        if !state.is_active {
            return Ok(None);
        }

        let (current_state, changed_since_last_sync) = match &self.last_state {
            [(path, old_state), _] | [_, (path, old_state)] if *path == subaction_path => {
                let s = old_state.load(Ordering::Relaxed);
                let ret = if state.changed_since_last_sync && state.current_state {
                    !s
                } else {
                    s
                };

                let changed = old_state
                    .compare_exchange(!ret, ret, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok();
                (ret, changed)
            }
            [(_, state1), (_, state2)] if subaction_path == xr::Path::NULL => {
                let s = state1.load(Ordering::Relaxed) || state2.load(Ordering::Relaxed);
                let ret = if state.changed_since_last_sync && state.current_state {
                    !s
                } else {
                    s
                };
                let cmpex = |state: &AtomicBool| {
                    state
                        .compare_exchange(!ret, ret, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                };
                let changed1 = cmpex(state1);
                let changed2 = cmpex(state2);
                (ret, changed1 || changed2)
            }
            _ => unreachable!(),
        };

        Ok(Some(xr::ActionState {
            current_state,
            changed_since_last_sync,
            last_change_time: state.last_change_time,
            is_active: true,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{tests::Fixture, ActionData};
    use openvr as vr;
    use fakexr::UserPath::*;

    macro_rules! get_toggle_action {
        ($fixture:expr, $handle:expr, $toggle_data:ident) => {
            let data = $fixture.input.openxr.session_data.get();
            let actions = data.input_data.get_loaded_actions().unwrap();
            let ActionData::Bool(BoolActionData { toggle_data, .. }) =
                actions.try_get_action($handle).unwrap()
            else {
                panic!("should be bool action");
            };

            let $toggle_data = toggle_data.as_ref().unwrap();
        };
    }

    macro_rules! get_dpad_action {
        ($fixture:expr, $handle:expr, $dpad_data:ident) => {
            let data = $fixture.input.openxr.session_data.get();
            let actions = data.input_data.get_loaded_actions().unwrap();
            let ActionData::Bool(BoolActionData { dpad_data, .. }) =
                actions.try_get_action($handle).unwrap()
            else {
                panic!("should be bool action");
            };

            let $dpad_data = dpad_data.as_ref().unwrap();
        };
    }

    macro_rules! get_grab_action {
        ($fixture:expr, $handle:expr, $grab_data:ident) => {
            let data = $fixture.input.openxr.session_data.get();
            let actions = data.input_data.get_loaded_actions().unwrap();
            let ActionData::Bool(BoolActionData { grab_data, .. }) =
                actions.try_get_action($handle).unwrap()
            else {
                panic!("should be bool action");
            };

            let $grab_data = grab_data.as_ref().unwrap();
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

        let grab = GrabBindingData::GRAB_THRESHOLD;
        let release = GrabBindingData::RELEASE_THRESHOLD;
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

        let left = f.get_input_source_handle(c"/user/hand/left");
        let right = f.get_input_source_handle(c"/user/hand/right");

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

        let grab = GrabBindingData::GRAB_THRESHOLD;
        let release = GrabBindingData::RELEASE_THRESHOLD;
        value_state_check(grab - 0.1, 1.0, LeftHand, false, false, line!());
        value_state_check(grab - 0.1, 1.0, RightHand, false, false, line!());

        value_state_check(grab, 1.0, LeftHand, true, true, line!());
        value_state_check(grab, 1.0, RightHand, true, true, line!());

        value_state_check(0.0, release, LeftHand, false, true, line!());
        value_state_check(0.0, 1.0, RightHand, true, false, line!());
    }

    #[test]
    fn toggle_button() {
        let f = Fixture::new();
        let set1 = f.get_action_set_handle(c"/actions/set1");
        let boolact = f.get_action_handle(c"/actions/set1/in/boolact");
        f.load_actions(c"actions_toggle.json");

        get_toggle_action!(f, boolact, toggle_data);

        fakexr::set_action_state(
            toggle_data.action.as_raw(),
            fakexr::ActionState::Bool(true),
            LeftHand,
        );

        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });

        let state = f.get_bool_state(boolact).unwrap();
        assert!(state.bActive);
        assert!(state.bState);
        assert!(state.bChanged);

        fakexr::set_action_state(
            toggle_data.action.as_raw(),
            fakexr::ActionState::Bool(false),
            LeftHand,
        );

        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });

        let state = f.get_bool_state(boolact).unwrap();
        assert!(state.bActive);
        assert!(state.bState);
        assert!(!state.bChanged);

        fakexr::set_action_state(
            toggle_data.action.as_raw(),
            fakexr::ActionState::Bool(true),
            LeftHand,
        );

        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });

        let state = f.get_bool_state(boolact).unwrap();
        assert!(state.bActive);
        assert!(!state.bState);
        assert!(state.bChanged);

        // no change across sync point
        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });

        let state = f.get_bool_state(boolact).unwrap();
        assert!(state.bActive);
        assert!(!state.bState);
        assert!(!state.bChanged);
    }

    #[test]
    fn toggle_button_per_hand() {
        let f = Fixture::new();
        let set1 = f.get_action_set_handle(c"/actions/set1");
        let boolact = f.get_action_handle(c"/actions/set1/in/boolact");
        let left = f.get_input_source_handle(c"/user/hand/left");
        let right = f.get_input_source_handle(c"/user/hand/right");

        f.load_actions(c"actions_toggle.json");
        get_toggle_action!(f, boolact, toggle_data);

        let act = toggle_data.action.as_raw();

        fakexr::set_action_state(act, false.into(), LeftHand);
        fakexr::set_action_state(act, false.into(), RightHand);
        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });

        let s_left = f.get_bool_state_hand(boolact, left).unwrap();
        assert!(s_left.bActive);
        assert!(!s_left.bState);
        assert!(!s_left.bChanged);

        let s_right = f.get_bool_state_hand(boolact, right).unwrap();
        assert!(s_right.bActive);
        assert!(!s_right.bState);
        assert!(!s_right.bChanged);

        fakexr::set_action_state(act, true.into(), LeftHand);
        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });

        let s_left = f.get_bool_state_hand(boolact, left).unwrap();
        assert!(s_left.bActive);
        assert!(s_left.bState);
        assert!(s_left.bChanged);

        let s_right = f.get_bool_state_hand(boolact, right).unwrap();
        assert!(s_right.bActive);
        assert!(!s_right.bState);
        assert!(!s_right.bChanged);

        fakexr::set_action_state(act, false.into(), LeftHand);
        fakexr::set_action_state(act, true.into(), RightHand);
        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });

        let s_left = f.get_bool_state_hand(boolact, left).unwrap();
        assert!(s_left.bActive);
        assert!(s_left.bState);
        assert!(!s_left.bChanged);

        let s_right = f.get_bool_state_hand(boolact, right).unwrap();
        assert!(s_right.bActive);
        assert!(s_right.bState);
        assert!(s_right.bChanged);
    }
}
