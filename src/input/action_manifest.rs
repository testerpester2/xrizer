use super::{knuckles::Knuckles, vive_controller::ViveWands, Input};
use crate::{
    openxr_data::{self, Hand, SessionData},
    vr,
};
use log::{debug, error, info, trace, warn};
use openxr as xr;
use serde::{
    de::{Error, IgnoredAny, Unexpected},
    Deserialize,
};
use slotmap::SecondaryMap;
use std::cell::{LazyCell, RefCell};
use std::collections::HashMap;
use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

impl<C: openxr_data::Compositor> Input<C> {
    pub(super) fn load_action_manifest(
        &self,
        session_data: &SessionData,
        manifest_path: &Path,
    ) -> Result<(), vr::EVRInputError> {
        match self.loaded_actions_path.get() {
            Some(p) => {
                assert_eq!(p, manifest_path);
            }
            None => self
                .loaded_actions_path
                .set(manifest_path.to_path_buf())
                .unwrap(),
        }

        let data = std::fs::read(manifest_path).map_err(|e| {
            error!("Failed to read manifest {}: {e}", manifest_path.display());
            vr::EVRInputError::VRInputError_InvalidParam
        })?;

        let manifest: ActionManifest = serde_json::from_slice(&data).map_err(|e| {
            error!("Failed to parse action manifest: {e}");
            vr::EVRInputError::VRInputError_InvalidParam
        })?;

        // TODO: support non english localization?
        let english = manifest
            .localization
            .and_then(|l| l.into_iter().find(|l| l.language_tag == "en_US"));

        let sets = load_action_sets(
            &self.openxr.instance,
            english.as_ref(),
            manifest.action_sets,
        )?;
        debug!("Loaded {} action sets.", sets.len());

        let mut actions = load_actions(
            &session_data.session,
            english.as_ref(),
            &sets,
            manifest.actions,
            self.openxr.left_hand.subaction_path,
            self.openxr.right_hand.subaction_path,
        )?;
        debug!("Loaded {} actions.", actions.len());

        // Games can mix legacy and normal input, and the legacy bindings are used for
        // WaitGetPoses, so attach the legacy set here as well.
        let legacy = session_data.input_data.legacy_actions.get_or_init(|| {
            super::LegacyActions::new(
                &self.openxr.instance,
                &session_data.session,
                self.openxr.left_hand.subaction_path,
                self.openxr.right_hand.subaction_path,
            )
        });

        self.load_bindings(
            manifest_path.parent().unwrap(),
            &sets,
            &mut actions,
            manifest.default_bindings,
            legacy,
        );

        let xr_sets: Vec<_> = sets.values().chain(std::iter::once(&legacy.set)).collect();
        session_data.session.attach_action_sets(&xr_sets).unwrap();

        // Transform actions and sets into maps
        // If the application has already requested the handle for an action/set, we need to
        // reuse the corresponding slot. Otherwise just create a new one.
        let mut set_guard = self.set_map.write().unwrap();
        let sets: SecondaryMap<_, _> = sets
            .into_iter()
            .map(|(set_name, set)| {
                // This function is only called when loading the action manifest, and most games
                // don't have a ton of actions, so a linear search through the map is probably fine.
                let key = set_guard
                    .iter()
                    .find_map(|(key, set_path)| (*set_path == set_name).then_some(key))
                    .unwrap_or_else(|| set_guard.insert(set_name));
                (key, set)
            })
            .collect();

        let mut act_guard = self.action_map.write().unwrap();
        let actions: SecondaryMap<_, _> = actions
            .into_iter()
            .map(|(name, action)| {
                let key = act_guard
                    .iter()
                    .find_map(|(key, super::Action { path })| (*path == name).then_some(key))
                    .unwrap_or_else(|| act_guard.insert(super::Action { path: name }));

                (key, action)
            })
            .collect();

        let loaded = super::LoadedActions { sets, actions };

        match session_data.input_data.loaded_actions.get() {
            Some(lock) => {
                *lock.write().unwrap() = loaded;
            }
            None => {
                session_data
                    .input_data
                    .loaded_actions
                    .set(RwLock::new(loaded))
                    .unwrap_or_else(|_| unreachable!());
            }
        }
        Ok(())
    }
}

/**
 * Structure for action manifests.
 * https://github.com/ValveSoftware/openvr/wiki/Action-manifest
 */

#[derive(Deserialize)]
struct ActionManifest {
    default_bindings: Vec<DefaultBindings>,
    action_sets: Vec<ActionSetJson>,
    actions: Vec<ActionJson>,
    localization: Option<Vec<Localization>>,
    // localization_files
}

#[derive(Deserialize)]
struct DefaultBindings {
    binding_url: PathBuf,
    controller_type: ControllerType,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ControllerType {
    ViveController,
    Knuckles,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Deserialize)]
struct ActionSetJson {
    #[serde(rename = "name")]
    path: String,
}

#[derive(Deserialize)]
struct ActionJson {
    #[serde(rename = "name")]
    path: String,
    #[serde(rename = "type")]
    ty: ActionType,
    #[serde(default, deserialize_with = "parse_skeleton")]
    skeleton: Option<Hand>,
}

fn parse_skeleton<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Hand>, D::Error> {
    let path: &str = Deserialize::deserialize(d)?;
    let Some(hand) = path.strip_prefix("/skeleton/hand") else {
        return Err(D::Error::invalid_value(
            Unexpected::Str(path),
            &"path starting with /skeleton/hand",
        ));
    };

    match hand {
        "/left" => Ok(Some(Hand::Left)),
        "/right" => Ok(Some(Hand::Right)),
        _ => Err(D::Error::invalid_value(
            Unexpected::Str(hand),
            &r#""/left" or "/right""#,
        )),
    }
}

#[derive(Deserialize)]
struct Localization {
    language_tag: String,
    #[serde(flatten)]
    localized_names: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "lowercase")]
enum ActionType {
    Boolean,
    Vector1,
    Vector2,
    Vibration,
    Pose,
    Skeleton,
}

fn load_action_sets(
    instance: &xr::Instance,
    english: Option<&Localization>,
    sets: Vec<ActionSetJson>,
) -> Result<HashMap<String, xr::ActionSet>, vr::EVRInputError> {
    let mut action_sets = HashMap::new();
    for ActionSetJson { path } in sets {
        let localized = english
            .and_then(|e| e.localized_names.get(&path))
            .unwrap_or(&path);

        let path = path.to_lowercase();
        // OpenXR does not like the "/actions/<set name>" format, so we need to strip the prefix
        let Some(xr_friendly_name) = path.strip_prefix("/actions/") else {
            error!("Action set {path} missing actions prefix.");
            return Err(vr::EVRInputError::VRInputError_InvalidParam);
        };

        trace!("Creating action set {xr_friendly_name} ({path:?}) (localized: {localized})");
        let set = instance
            .create_action_set(&xr_friendly_name, localized, 0)
            .map_err(|e| {
                error!("Failed to create action set: {e}");
                vr::EVRInputError::VRInputError_InvalidParam
            })?;

        action_sets.insert(path, set);
    }
    Ok(action_sets)
}

type LoadedActionDataMap = HashMap<String, super::ActionData>;

#[track_caller]
fn find_action(actions: &LoadedActionDataMap, name: &str) -> bool {
    let ret = actions.contains_key(name);
    if !ret {
        let caller = std::panic::Location::caller();
        warn!("Couldn't find action {name}, skipping ({})", caller.line());
    }
    ret
}

fn load_actions(
    session: &xr::Session<xr::vulkan::Vulkan>,
    english: Option<&Localization>,
    sets: &HashMap<String, xr::ActionSet>,
    actions: Vec<ActionJson>,
    left_hand: xr::Path,
    right_hand: xr::Path,
) -> Result<HashMap<String, super::ActionData>, vr::EVRInputError> {
    let mut ret = HashMap::with_capacity(actions.len());
    for ActionJson { path, ty, skeleton } in actions {
        let localized = english
            .and_then(|e| e.localized_names.get(&path))
            .map(|s| s.as_str());

        let path = path.to_lowercase();
        let set_end_idx = path.match_indices('/').nth(2).unwrap().0;
        let set_name = &path[0..set_end_idx];
        let xr_friendly_name = path.rsplit_once('/').unwrap().1;
        let localized = localized.unwrap_or(xr_friendly_name);

        trace!("Creating action {xr_friendly_name} (localized: {localized}) in set {set_name:?}");
        let set = &sets[set_name];

        fn create_action<T: xr::ActionTy>(
            set: &xr::ActionSet,
            name: &str,
            localized: &str,
            paths: &[xr::Path],
        ) -> xr::Result<xr::Action<T>> {
            set.create_action(name, localized, paths).or_else(|err| {
                // If we get a duplicated localized name, just unduplicate it and try again
                if err == xr::sys::Result::ERROR_LOCALIZED_NAME_DUPLICATED {
                    let localized = format!("{localized} (copy)");
                    create_action(set, name, &localized, paths)
                } else {
                    Err(err)
                }
            })
        }

        let paths = &[left_hand, right_hand];
        use super::ActionData::*;
        let action = match ty {
            ActionType::Boolean => Bool(super::BoolActionData {
                action: create_action::<bool>(&set, &xr_friendly_name, localized, paths).unwrap(),
                dpad_data: None,
            }),
            ActionType::Vector1 => Vector1 {
                action: create_action::<f32>(&set, &xr_friendly_name, localized, paths).unwrap(),
                last_value: super::AtomicF32::new(0.0),
            },
            ActionType::Vector2 => Vector2 {
                action: create_action::<xr::Vector2f>(&set, &xr_friendly_name, localized, paths)
                    .unwrap(),
                last_value: (super::AtomicF32::new(0.0), super::AtomicF32::new(0.0)),
            },
            ActionType::Pose => {
                let action =
                    create_action::<xr::Posef>(&set, &xr_friendly_name, localized, paths).unwrap();
                let left_space = action
                    .create_space(session, left_hand, xr::Posef::IDENTITY)
                    .unwrap();
                let right_space = action
                    .create_space(session, right_hand, xr::Posef::IDENTITY)
                    .unwrap();
                Pose {
                    action,
                    left_space,
                    right_space,
                }
            }
            ActionType::Skeleton => {
                let hand = skeleton.expect("Got skeleton action without path");
                let action =
                    create_action::<xr::Posef>(&set, &xr_friendly_name, localized, paths).unwrap();
                let space = action
                    .create_space(
                        session,
                        match hand {
                            Hand::Left => left_hand,
                            Hand::Right => right_hand,
                        },
                        xr::Posef::IDENTITY,
                    )
                    .unwrap();

                let hand_tracker = match session.create_hand_tracker(match hand {
                    Hand::Left => xr::Hand::LEFT,
                    Hand::Right => xr::Hand::RIGHT,
                }) {
                    Ok(t) => Some(t),
                    Err(
                        xr::sys::Result::ERROR_EXTENSION_NOT_PRESENT
                        | xr::sys::Result::ERROR_FEATURE_UNSUPPORTED,
                    ) => None,
                    Err(other) => panic!("Creating hand tracker failed: {other:?}"),
                };

                Skeleton {
                    action,
                    space,
                    hand,
                    hand_tracker,
                }
            }
            ActionType::Vibration => Haptic(
                create_action::<xr::Haptic>(&set, &xr_friendly_name, localized, paths).unwrap(),
            ),
        };
        ret.insert(path, action);
    }
    Ok(ret)
}

/**
 * Structure for binding files
 */

#[derive(Deserialize)]
struct Bindings {
    bindings: HashMap<String, ActionSetBinding>,
}

#[derive(Deserialize)]
struct ActionSetBinding {
    sources: Vec<ActionBinding>,
    poses: Option<Vec<SimpleActionBinding>>,
    haptics: Option<Vec<SimpleActionBinding>>,
    skeleton: Option<Vec<SkeletonActionBinding>>,
}

#[derive(Deserialize)]
struct SimpleActionBinding {
    output: String,
    path: String,
}

#[derive(Deserialize)]
struct SkeletonActionBinding {
    output: String,
    #[serde(deserialize_with = "path_to_skeleton")]
    path: Hand,
}

fn path_to_skeleton<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Hand, D::Error> {
    let path: &str = Deserialize::deserialize(d)?;
    match path {
        "/user/hand/left/input/skeleton/left" => Ok(Hand::Left),
        "/user/hand/right/input/skeleton/right" => Ok(Hand::Right),
        other => Err(D::Error::invalid_value(
            Unexpected::Str(other),
            &"/user/hand/left/input/skeleton/left or /user/hand/right/input/skeleton/right",
        )),
    }
}

#[derive(Deserialize, Debug)]
struct ActionBindingOutput {
    output: String,
}

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
enum ActionBinding {
    None(IgnoredAny),
    Button {
        path: String,
        inputs: ButtonInput,
        #[serde(rename = "parameters")]
        _parameters: Option<ButtonParameters>,
    },
    Dpad {
        path: String,
        inputs: DpadInput,
        parameters: Option<DpadParameters>,
    },
    Trigger {
        path: String,
        inputs: TriggerInput,
    },
    ScalarConstant {
        path: String,
        inputs: ScalarConstantInput,
    },
    ForceSensor {
        path: String,
        inputs: ForceSensorInput,
        #[serde(rename = "parameters")]
        _parameters: Option<ForceSensorParameters>,
    },
    Grab {
        path: String,
        inputs: GrabInput,
        #[serde(rename = "parameters")]
        _parameters: Option<GrabParameters>,
    },
    Scroll {
        path: String,
        inputs: ScrollInput,
        parameters: Option<ScrollParameters>,
    },
    Trackpad(Vector2Mode),
    Joystick(Vector2Mode),
}

#[derive(Deserialize)]
struct ButtonInput {
    click: Option<ActionBindingOutput>,
    double: Option<ActionBindingOutput>,
}

#[derive(Deserialize)]
struct ButtonParameters {
    #[serde(rename = "force_input")]
    _force_input: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DpadInput {
    east: Option<ActionBindingOutput>,
    south: Option<ActionBindingOutput>,
    north: Option<ActionBindingOutput>,
    west: Option<ActionBindingOutput>,
    center: Option<ActionBindingOutput>,
}

#[derive(Deserialize)]
#[serde(default)]
struct DpadParameters {
    sub_mode: DpadSubMode,
    #[serde(deserialize_with = "parse_pct")]
    deadzone_pct: u8,
    #[serde(deserialize_with = "parse_pct")]
    overlap_pct: u8,
    #[serde(deserialize_with = "as_bool")]
    sticky: bool,
}

impl Default for DpadParameters {
    fn default() -> Self {
        Self {
            sub_mode: DpadSubMode::Touch,
            deadzone_pct: 50,
            overlap_pct: 50,
            sticky: false,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum DpadSubMode {
    Click,
    Touch,
}

fn parse_pct<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u8, D::Error> {
    let val: &str = Deserialize::deserialize(d)?;
    u8::from_str_radix(val, 10).map_err(|e| {
        D::Error::invalid_value(Unexpected::Str(val), &format!("a valid u8 ({e})").as_str())
    })
}

fn as_bool<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    let val: &str = Deserialize::deserialize(d)?;
    match val {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(D::Error::invalid_value(
            Unexpected::Str(other),
            &"true or false",
        )),
    }
}

#[derive(Deserialize)]
struct TriggerInput {
    pull: Option<ActionBindingOutput>,
    touch: Option<ActionBindingOutput>,
    click: Option<ActionBindingOutput>,
}

#[derive(Deserialize)]
struct ScalarConstantInput {
    value: ActionBindingOutput,
}

#[derive(Deserialize)]
struct ForceSensorInput {
    force: ActionBindingOutput,
}

#[derive(Deserialize)]
struct ForceSensorParameters {
    #[allow(unused)]
    haptic_amplitude: Option<String>,
}

#[derive(Deserialize)]
struct GrabInput {
    grab: ActionBindingOutput,
}

#[derive(Deserialize)]
struct GrabParameters {
    #[serde(
        default,
        rename = "value_hold_threshold",
        deserialize_with = "grab_threshold"
    )]
    _value_hold_threshold: Option<f32>,
    #[serde(
        default,
        rename = "value_release_threshold",
        deserialize_with = "grab_threshold"
    )]
    _value_release_threshold: Option<f32>,
}

fn grab_threshold<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f32>, D::Error> {
    let val: &str = Deserialize::deserialize(d)?;
    match val.parse() {
        Ok(v) => Ok(Some(v)),
        Err(e) => Err(D::Error::custom(e)),
    }
}

#[derive(Deserialize)]
struct ScrollInput {
    scroll: ActionBindingOutput,
}

#[derive(Deserialize)]
struct ScrollParameters {
    scroll_mode: Option<String>,
    smooth_scroll_multiplier: Option<String>, // float
}

#[derive(Deserialize)]
struct Vector2Mode {
    path: String,
    inputs: Vector2Input,
}

#[derive(Deserialize)]
struct Vector2Input {
    position: Option<ActionBindingOutput>,
    click: Option<ActionBindingOutput>,
    touch: Option<ActionBindingOutput>,
}

/// Call a generic function with each supported interaction profile.
/// The profile is provided as a type parameter named P.
macro_rules! for_each_profile {
    (<
        $($lifetimes:lifetime),*
        $(,$generic_name:ident $(: $generic_bound:path)?)*
    > ($($arg_name:ident: $arg_ty:ty),*) $block:block) => {{
        struct S<$($lifetimes,)* $($generic_name $(: $generic_bound)?),*> {
            $(
                $arg_name: $arg_ty
            ),*
        }

        impl<$($lifetimes,)* $($generic_name $(: $generic_bound)?),*> crate::input::action_manifest::ForEachProfile
            for S<$($lifetimes,)* $($generic_name),*> {
            fn call<P: InteractionProfile>(&mut self) {
                let S {
                    $($arg_name),*
                } = self;
                $block
            }
        }

        crate::input::action_manifest::for_each_profile_fn(S { $($arg_name),* });
    }};
}
pub(super) use for_each_profile;

impl<C: openxr_data::Compositor> Input<C> {
    fn load_bindings<'a>(
        &self,
        parent_path: &Path,
        action_sets: &HashMap<String, xr::ActionSet>,
        actions: &mut LoadedActionDataMap,
        bindings: Vec<DefaultBindings>,
        legacy_actions: &super::LegacyActions,
    ) {
        let mut it: Box<dyn Iterator<Item = DefaultBindings>> = Box::new(bindings.into_iter());
        while let Some(DefaultBindings {
            binding_url,
            controller_type,
        }) = it.next()
        {
            let load_bindings = || {
                let bindings_path = parent_path.join(binding_url);
                debug!(
                    "Reading bindings for {controller_type:?} (at {})",
                    bindings_path.display()
                );

                let data = std::fs::read(bindings_path)
                    .inspect_err(|e| error!("Couldn't load bindings for {controller_type:?}: {e}"))
                    .ok()?;

                let Bindings { bindings } = serde_json::from_slice(&data)
                    .inspect_err(|e| {
                        error!("Failed to parse bindings for {controller_type:?}: {e}")
                    })
                    .ok()?;

                Some(bindings)
            };
            macro_rules! load_bindings {
                ($ty:ty) => {
                    if let Some(bindings) = load_bindings() {
                        self.load_bindings_for_profile::<$ty>(
                            action_sets,
                            actions,
                            legacy_actions,
                            &bindings,
                        )
                    }
                };
            }
            match controller_type {
                ControllerType::ViveController => load_bindings!(ViveWands),
                ControllerType::Knuckles => load_bindings!(Knuckles),
                ControllerType::Unknown(ref other) => {
                    warn!("Ignoring bindings for unknown profile {other}")
                }
            }

            it = Box::new(it.skip_while(move |b| {
                if b.controller_type == controller_type {
                    debug!("skipping bindings in {:?}", b.binding_url);
                    true
                } else {
                    false
                }
            }));
        }
    }

    fn load_bindings_for_profile<P: InteractionProfile>(
        &self,
        action_sets: &HashMap<String, xr::ActionSet>,
        actions: &mut LoadedActionDataMap,
        legacy_actions: &super::LegacyActions,
        bindings: &HashMap<String, ActionSetBinding>,
    ) {
        info!("loading bindings for {}", P::PROFILE_PATH);
        let legal_paths = P::legal_paths();
        let translate_map = P::TRANSLATE_MAP;
        let path_translator = |path: &str| {
            let mut translated = path.to_string();
            for PathTranslation { from, to, stop } in translate_map {
                if translated.find(from).is_some() {
                    translated = translated.replace(from, to);
                    if *stop {
                        break;
                    }
                }
            }
            trace!("translated {path} to {translated}");
            if !legal_paths.contains(&translated) {
                Err(format!("Action for invalid path {translated}, ignoring"))
            } else {
                Ok(translated)
            }
        };

        let mut xr_bindings = Vec::new();
        for (action_set_name, bindings) in bindings.into_iter() {
            let Some(set) = action_sets.get(action_set_name) else {
                warn!("Action set {action_set_name} missing.");
                continue;
            };

            if let Some(bindings) = &bindings.haptics {
                xr_bindings.extend(handle_haptic_bindings(
                    &self.openxr.instance,
                    path_translator,
                    actions,
                    bindings,
                ));
            }

            if let Some(bindings) = &bindings.poses {
                xr_bindings.extend(handle_pose_bindings(
                    &self.openxr.instance,
                    path_translator,
                    actions,
                    bindings,
                ));
            }

            if let Some(bindings) = &bindings.skeleton {
                xr_bindings.extend(handle_skeleton_bindings(
                    &self.openxr.instance,
                    actions,
                    bindings,
                ));
            }

            xr_bindings.extend(handle_sources(
                &self.openxr.instance,
                path_translator,
                actions,
                action_set_name,
                set,
                &bindings.sources,
            ));
        }

        // Workaround weird closure lifetime quirks.
        const fn constrain<F>(f: F) -> F
        where
            F: for<'a> Fn(&'a str) -> openxr::Path,
        {
            f
        }
        let stp = constrain(|s| self.openxr.instance.string_to_path(s).unwrap());
        let legacy_bindings = P::legacy_bindings(stp, &legacy_actions);
        let profile_path = stp(P::PROFILE_PATH);

        let bindings: Vec<xr::Binding<'_>> = xr_bindings
            .into_iter()
            .map(|(name, path)| {
                use super::ActionData::*;
                match &actions[&name] {
                    Bool(data) => xr::Binding::new(&data.action, path),
                    Vector1 { action, .. } => xr::Binding::new(&action, path),
                    Vector2 { action, .. } => xr::Binding::new(&action, path),
                    Haptic(action) => xr::Binding::new(&action, path),
                    Skeleton { action, .. } => xr::Binding::new(&action, path),
                    Pose { action, .. } => xr::Binding::new(&action, path),
                }
            })
            .chain(legacy_bindings)
            .collect();

        self.openxr
            .instance
            .suggest_interaction_profile_bindings(profile_path, &bindings)
            .unwrap();
        debug!(
            "suggested {} bindings for {}",
            bindings.len(),
            P::PROFILE_PATH
        );
    }
}

/// Returns a tuple of a parent action index and a path for its bindng
fn handle_dpad_action(
    string_to_path: impl Fn(&str) -> Option<xr::Path>,
    parent_path: &str,
    action_set_name: &str,
    action_set: &xr::ActionSet,
    actions: &mut LoadedActionDataMap,
    DpadInput {
        east,
        south,
        north,
        west,
        center,
    }: &DpadInput,
    parameters: Option<&DpadParameters>,
) -> Vec<(String, xr::Path)> {
    // Would love to use the dpad extension here, but it doesn't seem to
    // support touch trackpad dpads.
    // TODO: actually take the deadzone and overlap into account

    // Workaround weird closure lifetime quirks.
    const fn constrain<F>(f: F) -> F
    where
        F: for<'a> Fn(&'a Option<ActionBindingOutput>, super::DpadDirection) -> Option<&'a str>,
    {
        f
    }
    let maybe_find_action = constrain(|a, direction| {
        let output = &a.as_ref()?.output;
        let ret = actions.contains_key(output);
        if !ret {
            warn!(
                "Couldn't find dpad action {} (for path {parent_path}, {direction:?})",
                output
            );
        }
        ret.then_some(output)
    });

    use super::DpadDirection::*;

    let bound_actions: Vec<(&str, super::DpadDirection)> = [
        (maybe_find_action(north, North), North),
        (maybe_find_action(east, East), East),
        (maybe_find_action(south, South), South),
        (maybe_find_action(west, West), West),
        (maybe_find_action(center, Center), Center),
    ]
    .into_iter()
    .flat_map(|(name, direction)| name.zip(Some(direction)))
    .collect();

    if bound_actions.is_empty() {
        warn!("Dpad mode, but no actions ({parent_path} in {action_set_name})");
        return vec![];
    }

    let parent_action_key = format!("{parent_path}-{action_set_name}");
    let actions = RefCell::new(actions);
    let created_actions = LazyCell::new(|| {
        get_dpad_parent(
            &string_to_path,
            parent_path,
            &parent_action_key,
            action_set_name,
            action_set,
            &actions,
            parameters,
        )
    });
    for (action_name, direction) in bound_actions {
        // Temporarily remove action to avoid double mutable reference
        let super::ActionData::Bool(mut data) = actions.borrow_mut().remove(action_name).unwrap()
        else {
            panic!("Expected bool action for dpad binding on {}", action_name);
        };

        if data.dpad_data.is_none() {
            let (parent_action, click_or_touch) = LazyCell::force(&created_actions);
            data.dpad_data = Some(super::DpadData {
                parent: parent_action.clone(),
                click_or_touch: click_or_touch.as_ref().map(|d| d.action.clone()),
                direction,
                last_state: false.into(),
            })
        }

        // Reinsert
        actions
            .borrow_mut()
            .insert(action_name.to_string(), super::ActionData::Bool(data));
    }

    let activator_binding = created_actions
        .1
        .as_ref()
        .map(|DpadActivatorData { key, binding, .. }| (key.clone(), *binding));
    let mut ret = vec![(parent_action_key, string_to_path(parent_path).unwrap())];
    if let Some(b) = activator_binding {
        ret.push(b);
    }
    ret
}

struct DpadActivatorData {
    key: String,
    action: xr::Action<bool>,
    binding: xr::Path,
}

fn get_dpad_parent(
    string_to_path: &impl Fn(&str) -> Option<xr::Path>,
    parent_path: &str,
    parent_action_key: &str,
    action_set_name: &str,
    action_set: &xr::ActionSet,
    actions: &RefCell<&mut LoadedActionDataMap>,
    parameters: Option<&DpadParameters>,
) -> (xr::Action<xr::Vector2f>, Option<DpadActivatorData>) {
    let mut actions = actions.borrow_mut();
    // Share parent actions that use the same action set and same bound path
    let parent_action = actions
        .entry(parent_action_key.to_string())
        .or_insert_with(|| {
            let clean_parent_path = parent_path.replace("/", "_");
            let parent_action_name = format!("xrizer-dpad-parent-{clean_parent_path}");
            let localized = format!("XRizer dpad parent ({parent_path})");
            let action = action_set
                .create_action::<xr::Vector2f>(&parent_action_name, &localized, &[])
                .unwrap();

            trace!("created new dpad parent ({parent_action_key})");

            super::ActionData::Vector2 {
                action,
                last_value: Default::default(),
            }
        });
    let super::ActionData::Vector2 {
        action: parent_action,
        ..
    } = parent_action
    else {
        unreachable!();
    };
    // Remove lifetime
    let parent_action = parent_action.clone();

    // Create our path to our parent click/touch, if such a path exists
    let (activator_binding_str, activator_binding_path) = parameters
        .as_ref()
        .and_then(|p| {
            let name = match p.sub_mode {
                DpadSubMode::Click => format!("{parent_path}/click"),
                DpadSubMode::Touch => format!("{parent_path}/touch"),
            };
            string_to_path(&name).map(|p| (name, p))
        })
        .unzip();

    let activator_key = activator_binding_str
        .as_ref()
        .map(|n| format!("{n}-{action_set_name}"));
    // Action only needs to exist if our path was successfully created
    let len = actions.len();
    let activator_action = activator_key.as_ref().map(|key| {
        let action = actions.entry(key.clone()).or_insert_with(|| {
            let dpad_activator_name = format!("xrizer-dpad-active{len}");
            let localized = format!("XRizer dpad active ({len})");

            super::ActionData::Bool(super::BoolActionData {
                action: action_set
                    .create_action(&dpad_activator_name, &localized, &[])
                    .unwrap(),
                dpad_data: None,
            })
        });

        let super::ActionData::Bool(super::BoolActionData { action, .. }) = action else {
            unreachable!();
        };
        action
    });
    // Remove lifetime
    let click_or_touch = activator_action.cloned();

    (
        parent_action,
        click_or_touch.map(|action| DpadActivatorData {
            key: activator_key.unwrap(),
            action,
            binding: activator_binding_path.unwrap(),
        }),
    )
}

fn translate_warn<'a>(action: &'a str) -> impl FnOnce(&String) + 'a {
    move |e| warn!("{e} ({action})")
}

fn handle_sources(
    instance: &xr::Instance,
    path_translator: impl Fn(&str) -> Result<String, String>,
    actions: &mut LoadedActionDataMap,
    action_set_name: &str,
    action_set: &xr::ActionSet,
    sources: &[ActionBinding],
) -> Vec<(String, xr::Path)> {
    fn try_get_binding(
        actions: &LoadedActionDataMap,
        instance: &xr::Instance,
        action_path: String,
        input_path: String,
        action_pattern: impl Fn(&super::ActionData),
        bindings: &mut Vec<(String, xr::Path)>,
    ) {
        if find_action(&actions, &action_path) {
            action_pattern(&actions[&action_path]);
            trace!("suggesting {input_path} for {action_path}");
            let binding_path = instance.string_to_path(&input_path).unwrap();
            bindings.push((action_path, binding_path));
        }
    }

    macro_rules! action_match {
        ($pat:pat, $($assert_msg:tt)*) => {
            |data| {
                assert!(
                    matches!(data, $pat),
                    $($assert_msg)*
                )
            }
        }
    }

    use super::ActionData::*;
    let mut bindings = Vec::new();
    for mode in sources {
        let mut try_get_bool_binding = |action_path: &str, input_path: String| {
            try_get_binding(
                actions,
                instance,
                action_path.to_string(),
                input_path,
                action_match!(
                    Bool(_) | Vector1 { .. },
                    "Action for {} should be a bool or float",
                    action_path
                ),
                &mut bindings,
            );
        };

        match mode {
            ActionBinding::None(_) => {}
            ActionBinding::Button {
                path,
                inputs,
                _parameters,
            } => {
                if let Some(ActionBindingOutput { output }) = &inputs.click {
                    let Ok(translated) = path_translator(&format!("{path}/click"))
                        .inspect_err(translate_warn(output))
                    else {
                        continue;
                    };

                    try_get_bool_binding(output, translated);
                }

                if let Some(ActionBindingOutput { output }) = &inputs.double {
                    warn!("Double click binding for {output} currently unsupported.");
                }
            }
            ActionBinding::Dpad {
                path,
                inputs,
                parameters,
            } => {
                let Ok(parent_translated) =
                    path_translator(path).inspect_err(translate_warn(&format!("{inputs:?}")))
                else {
                    continue;
                };
                let data = handle_dpad_action(
                    |s| {
                        path_translator(s)
                            .inspect_err(translate_warn("<dpad action>"))
                            .ok()
                            .map(|s| instance.string_to_path(&s).unwrap())
                    },
                    &parent_translated,
                    action_set_name,
                    action_set,
                    actions,
                    inputs,
                    parameters.as_ref(),
                );

                bindings.extend(data);
            }
            ActionBinding::Trigger {
                path,
                inputs: TriggerInput { pull, touch, click },
            } => {
                let suffixes_and_outputs = [("pull", pull), ("touch", touch), ("click", click)]
                    .into_iter()
                    .filter_map(|(sfx, input)| Some(sfx).zip(input.as_ref().map(|i| &i.output)));
                for (suffix, output) in suffixes_and_outputs {
                    let Ok(translated) = path_translator(&format!("{path}/{suffix}"))
                        .inspect_err(translate_warn(output))
                    else {
                        continue;
                    };

                    try_get_binding(
                        actions,
                        instance,
                        output.clone(),
                        translated,
                        action_match!(
                            Bool(_) | Vector1 { .. },
                            "Trigger action should be a bool or float"
                        ),
                        &mut bindings,
                    );
                }
            }
            ActionBinding::ScalarConstant {
                path,
                inputs:
                    ScalarConstantInput {
                        value: ActionBindingOutput { output },
                    },
            } => {
                let vpath = format!("{path}/value");
                let Ok(translated) = path_translator(&vpath)
                    .or_else(|_| {
                        trace!("Invalid scalar constant path {vpath}, trying click");
                        path_translator(&format!("{path}/click"))
                    })
                    .inspect_err(translate_warn(output))
                else {
                    continue;
                };

                try_get_binding(
                    actions,
                    instance,
                    output.to_string(),
                    translated,
                    action_match! {
                        Vector1 { .. },
                        "Expected Vector1 action for {}",
                        output
                    },
                    &mut bindings,
                )
            }
            ActionBinding::ForceSensor {
                path,
                inputs:
                    ForceSensorInput {
                        force: ActionBindingOutput { output },
                    },
                _parameters,
            } => {
                let Ok(translated) =
                    path_translator(&format!("{path}/force")).inspect_err(translate_warn(output))
                else {
                    continue;
                };

                try_get_binding(
                    actions,
                    instance,
                    output.to_string(),
                    translated,
                    action_match! {
                        Vector1 { .. },
                        "Expected Vector1 action for {}",
                        output
                    },
                    &mut bindings,
                );
            }
            ActionBinding::Grab {
                path,
                inputs:
                    GrabInput {
                        grab: ActionBindingOutput { output },
                    },
                _parameters,
            } => {
                let Ok(translated) =
                    path_translator(&format!("{path}/grab")).inspect_err(translate_warn(output))
                else {
                    continue;
                };

                try_get_binding(
                    actions,
                    instance,
                    output.to_string(),
                    translated,
                    action_match!(
                        Bool(_) | Vector1 { .. },
                        "Grab action {} should be a bool or float",
                        output
                    ),
                    &mut bindings,
                );
            }
            ActionBinding::Scroll {
                path: _,
                inputs,
                parameters: _,
            } => {
                warn!("Got scroll binding for input {}, but these are currently unimplemented, skipping", inputs.scroll.output);
            }
            ActionBinding::Trackpad(data) | ActionBinding::Joystick(data) => {
                let Vector2Mode { path, inputs } = data;
                let Ok(translated) =
                    path_translator(path).inspect_err(translate_warn("<vector2 input>"))
                else {
                    continue;
                };

                let Vector2Input {
                    position,
                    click,
                    touch,
                } = inputs;

                if let Some((output, click_path)) = click.as_ref().and_then(|b| {
                    Some(&b.output).zip(
                        path_translator(&format!("{translated}/click"))
                            .inspect_err(translate_warn(&b.output))
                            .ok(),
                    )
                }) {
                    try_get_bool_binding(output, click_path);
                }

                if let Some((output, touch_path)) = touch.as_ref().and_then(|b| {
                    Some(&b.output).zip(
                        path_translator(&format!("{translated}/touch"))
                            .inspect_err(translate_warn(&b.output))
                            .ok(),
                    )
                }) {
                    try_get_bool_binding(output, touch_path);
                }

                if let Some(position) = position.as_ref() {
                    try_get_binding(
                        actions,
                        instance,
                        position.output.clone(),
                        translated.clone(),
                        action_match!(
                            Vector2 { .. },
                            "Expected Vector2 action for {}",
                            position.output
                        ),
                        &mut bindings,
                    );
                }
            }
        }
    }
    bindings
}

fn handle_skeleton_bindings(
    instance: &xr::Instance,
    actions: &LoadedActionDataMap,
    bindings: &[SkeletonActionBinding],
) -> Vec<(String, xr::Path)> {
    let mut ret = Vec::new();
    for SkeletonActionBinding { output, path } in bindings {
        if !find_action(actions, output) {
            continue;
        };
        let path = match path {
            Hand::Left => {
                trace!("suggested left grip for {output}");
                instance
                    .string_to_path("/user/hand/left/input/grip/pose")
                    .unwrap()
            }
            Hand::Right => {
                trace!("suggested right grip for {output}");
                instance
                    .string_to_path("/user/hand/right/input/grip/pose")
                    .unwrap()
            }
        };

        assert!(matches!(
            actions[output],
            super::ActionData::Skeleton { .. }
        ));
        ret.push((output.clone(), path));
    }

    ret
}

fn handle_haptic_bindings(
    instance: &xr::Instance,
    path_translator: impl Fn(&str) -> Result<String, String>,
    actions: &LoadedActionDataMap,
    bindings: &[SimpleActionBinding],
) -> Vec<(String, xr::Path)> {
    let mut ret = Vec::new();

    for SimpleActionBinding { output, path } in bindings {
        let Ok(translated) = path_translator(&path).inspect_err(translate_warn(output)) else {
            continue;
        };
        if !find_action(actions, output) {
            continue;
        };

        assert!(
            matches!(actions[output], super::ActionData::Haptic(_)),
            "expected haptic action for haptic binding {}, got {}",
            translated,
            output
        );
        let xr_path = instance.string_to_path(&translated).unwrap();
        ret.push((output.clone(), xr_path));
    }

    ret
}

fn handle_pose_bindings(
    instance: &xr::Instance,
    path_translator: impl Fn(&str) -> Result<String, String>,
    actions: &LoadedActionDataMap,
    bindings: &[SimpleActionBinding],
) -> Vec<(String, xr::Path)> {
    let mut ret = Vec::new();

    for SimpleActionBinding { output, path } in bindings {
        let Ok(translated) = path_translator(&path).inspect_err(translate_warn(output)) else {
            continue;
        };

        if !find_action(actions, output) {
            continue;
        };

        let xr_path = instance.string_to_path(&translated).unwrap();
        assert!(matches!(actions[output], super::ActionData::Pose { .. }));

        ret.push((output.clone(), xr_path));
    }

    ret
}

pub(super) struct PathTranslation {
    pub from: &'static str,
    pub to: &'static str,
    pub stop: bool,
}

pub(super) trait InteractionProfile {
    const PROFILE_PATH: &'static str;
    /// Corresponds to Prop_ModelNumber_String
    /// Can be pulled from a SteamVR System Report
    const MODEL: &'static CStr;
    /// Corresponds to Prop_ControllerType_String
    /// Can be pulled from a SteamVR System Report
    const OPENVR_CONTROLLER_TYPE: &'static CStr;
    const TRANSLATE_MAP: &'static [PathTranslation];

    fn legal_paths() -> Box<[String]>;
    fn legacy_bindings(
        string_to_path: impl for<'a> Fn(&'a str) -> xr::Path,
        actions: &super::LegacyActions,
    ) -> Vec<xr::Binding>;
}

pub(super) trait ForEachProfile {
    fn call<T: InteractionProfile>(&mut self);
}

/// Add all supported interaction profiles here.
pub(super) fn for_each_profile_fn<F: ForEachProfile>(mut f: F) {
    f.call::<super::vive_controller::ViveWands>();
    f.call::<super::knuckles::Knuckles>();
    f.call::<super::simple_controller::SimpleController>();
}
