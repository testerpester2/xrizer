use openxr as xr;
use std::f32::consts::{FRAC_PI_4, PI};
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct BoolActionData {
    pub action: xr::Action<bool>,
    pub dpad_data: Option<DpadData>,
    pub grab_data: Option<GrabBindingData>,
}

impl BoolActionData {
    pub fn new(action: xr::Action<bool>) -> Self {
        Self {
            action,
            dpad_data: None,
            grab_data: None,
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
