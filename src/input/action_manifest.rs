use super::Input;
use crate::{
    openxr_data::{self, Hand, SessionData},
    vr,
};
use log::{debug, error, info, trace, warn};
use openxr as xr;
use serde::{
    de::{Error, Unexpected},
    Deserialize,
};
use slotmap::SecondaryMap;
use std::collections::HashMap;
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

        let actions = load_actions(
            &session_data.session,
            english.as_ref(),
            &sets,
            manifest.actions,
            self.openxr.left_hand.path,
            self.openxr.right_hand.path,
        )?;
        debug!("Loaded {} actions.", actions.len());

        // Games can mix legacy and normal input, and the legacy bindings are used for
        // WaitGetPoses, so attach the legacy set here as well.
        let legacy = session_data.input_data.legacy_actions.get_or_init(|| {
            super::LegacyActions::new(
                &self.openxr.instance,
                &session_data.session,
                self.openxr.left_hand.path,
                self.openxr.right_hand.path,
            )
        });

        self.load_bindings(
            manifest_path.parent().unwrap(),
            &sets,
            &actions,
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
            .map(|LoadedActionInfo { name, action }| {
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

// https://github.com/ValveSoftware/openvr/wiki/Action-manifest
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
    OculusTouch,
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

struct LoadedActionInfo {
    name: String,
    action: super::ActionData,
}

fn load_actions(
    session: &xr::Session<xr::vulkan::Vulkan>,
    english: Option<&Localization>,
    sets: &HashMap<String, xr::ActionSet>,
    actions: Vec<ActionJson>,
    left_hand: xr::Path,
    right_hand: xr::Path,
) -> Result<Vec<LoadedActionInfo>, vr::EVRInputError> {
    let mut ret = Vec::new();
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
        use super::ActionData::*;

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
        let action = match ty {
            ActionType::Boolean => {
                Bool(create_action::<bool>(&set, &xr_friendly_name, localized, paths).unwrap())
            }
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
                let hand = skeleton.unwrap();
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

                Skeleton {
                    action,
                    space,
                    hand,
                }
            }
            ActionType::Vibration => Haptic(
                create_action::<xr::Haptic>(&set, &xr_friendly_name, localized, paths).unwrap(),
            ),
        };
        ret.push(LoadedActionInfo { name: path, action });
    }
    Ok(ret)
}

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
            &"left or right hand skeleton path",
        )),
    }
}

#[derive(Deserialize)]
struct ActionBinding {
    mode: ActionMode,
    path: String,
    inputs: ActionInput,
    parameters: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum ActionMode {
    Dpad,
    Button,
    Trigger,
    Trackpad,
    Joystick,
    None,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ActionInput {
    Button {
        click: ActionBindingInput,
    },
    Trigger {
        pull: ActionBindingInput,
    },
    Vector2 {
        position: ActionBindingInput,
    },
    Dpad {
        east: Option<ActionBindingInput>,
        south: Option<ActionBindingInput>,
        north: ActionBindingInput, // Assuming a dpad binding has at least the north component,
        // potentially a dangerous assumption!
        west: Option<ActionBindingInput>,
        center: Option<ActionBindingInput>,
    },
    Unknown(serde_json::Value),
}

#[derive(Deserialize, Debug)]
struct ActionBindingInput {
    output: String,
}

/// Call a generic function with each supported interaction profile.
/// All supported interaction profiles should be added here.
macro_rules! for_each_profile {
    ($fn:ident($($arg:expr),*)) => {{
        $fn::<crate::input::vive_controller::ViveWands>($($arg),*);
        $fn::<crate::input::simple_controller::SimpleController>($($arg),*);
    }};
    ($pfx:ident.$fn:ident($($arg:expr),*$(,)?)) => {{
        $pfx.$fn::<crate::input::vive_controller::ViveWands>($($arg),*);
        $pfx.$fn::<crate::input::simple_controller::SimpleController>($($arg),*);
    }}
}
pub(super) use for_each_profile;

impl<C: openxr_data::Compositor> Input<C> {
    fn load_bindings<'a>(
        &self,
        parent_path: &Path,
        action_sets: &HashMap<String, xr::ActionSet>,
        actions: &[LoadedActionInfo],
        bindings: Vec<DefaultBindings>,
        legacy_actions: &super::LegacyActions,
    ) {
        if let Some(DefaultBindings {
            binding_url,
            controller_type,
        }) = bindings
            .iter()
            .find(|b| b.controller_type == ControllerType::ViveController)
        {
            let bindings_path = parent_path.join(binding_url);
            debug!(
                "Reading bindings for {controller_type:?} (at {})",
                bindings_path.display()
            );

            let Ok(data) = std::fs::read(bindings_path)
                .inspect_err(|e| error!("Couldn't load bindings for {controller_type:?}: {e}"))
            else {
                return;
            };

            let Ok(Bindings { bindings }) = serde_json::from_slice(&data)
                .inspect_err(|e| error!("Failed to parse bindings for {controller_type:?}: {e}"))
            else {
                return;
            };

            for_each_profile! {
                self.load_bindings_for_profile(
                    action_sets,
                    actions,
                    legacy_actions,
                    &bindings,
                )
            }
        }
    }

    fn load_bindings_for_profile<P: InteractionProfile>(
        &self,
        action_sets: &HashMap<String, xr::ActionSet>,
        actions: &[LoadedActionInfo],
        legacy_actions: &super::LegacyActions,
        bindings: &HashMap<String, ActionSetBinding>,
    ) {
        use super::ActionData::*;

        info!("loading bindings for {}", P::PROFILE_PATH);
        let legal_paths = P::legal_paths();
        let translate_map = P::TRANSLATE_MAP;
        let path_translator = |path: &str| {
            let mut translated = path.to_string();
            for PathTranslation { from, to } in translate_map {
                if translated.find(from).is_some() {
                    translated = translated.replace(from, to);
                    break;
                }
            }
            trace!("translated {path} to {translated}");
            if !legal_paths.contains(&translated) {
                warn!("Action for invalid path {translated}, ignoring");
                None
            } else {
                Some(translated)
            }
        };

        macro_rules! find_action {
            ($name:expr) => {{
                let Some(action) = actions.iter().find(|a| a.name == *$name) else {
                    warn!("Couldn't find action {}, skipping", $name);
                    continue;
                };
                action
            }};
        }

        let mut xr_bindings = Vec::new();
        for (action_set_name, bindings) in bindings.into_iter() {
            if !action_sets.contains_key(action_set_name) {
                warn!("Action set {action_set_name} missing.");
                continue;
            }

            for SimpleActionBinding { output, path } in
                bindings.haptics.iter().flat_map(|p| p.iter())
            {
                let Some(translated) = path_translator(&path) else {
                    continue;
                };
                let xr_path = self.openxr.instance.string_to_path(&translated).unwrap();
                let action = find_action!(output);
                let Haptic(action) = &action.action else {
                    panic!(
                        "expected haptic action for haptic binding {}, got {}",
                        translated, output
                    );
                };

                xr_bindings.push(xr::Binding::new(action, xr_path));
            }

            for SimpleActionBinding { output, path } in bindings.poses.iter().flat_map(|p| p.iter())
            {
                let Some(translated) = path_translator(&path) else {
                    continue;
                };
                let xr_path = self.openxr.instance.string_to_path(&translated).unwrap();

                let action = find_action!(output);
                let binding = match &action.action {
                    Pose { action, .. } => xr::Binding::new(action, xr_path),
                    _ => unreachable!(),
                };

                xr_bindings.push(binding);
            }

            for SkeletonActionBinding { output, path } in
                bindings.skeleton.iter().flat_map(|s| s.iter())
            {
                let path = match path {
                    Hand::Left => {
                        trace!("suggested left grip for {output}");
                        self.openxr
                            .instance
                            .string_to_path("/user/hand/left/input/grip/pose")
                            .unwrap()
                    }
                    Hand::Right => {
                        trace!("suggested right grip for {output}");
                        self.openxr
                            .instance
                            .string_to_path("/user/hand/right/input/grip/pose")
                            .unwrap()
                    }
                };
                let action = find_action!(output);
                let binding = match &action.action {
                    Skeleton { action, .. } => xr::Binding::new(action, path),
                    _ => unreachable!(),
                };

                xr_bindings.push(binding);
            }

            for ActionBinding {
                mode,
                path,
                inputs,
                parameters,
            } in &bindings.sources
            {
                if matches!(mode, ActionMode::None) {
                    continue;
                }

                let Some(translated) = path_translator(&path) else {
                    continue;
                };
                let xr_path = || self.openxr.instance.string_to_path(&translated).unwrap();

                match mode {
                    ActionMode::None => unreachable!(),
                    ActionMode::Button => {
                        let ActionInput::Button { click } = inputs else {
                            error!(
                                "Input for button action for {path} in {action_set_name} is wrong."
                            );
                            continue;
                        };

                        let action = find_action!(click.output);
                        let binding = match &action.action {
                            Bool(b) => xr::Binding::new(b, xr_path()),
                            Vector1 { action, .. } => xr::Binding::new(action, xr_path()),
                            _ => {
                                panic!("Action for {} should be a bool or float", click.output)
                            }
                        };
                        trace!("suggesting {translated} for {}", click.output);
                        xr_bindings.push(binding);
                    }
                    ActionMode::Trigger => {
                        let output = match inputs {
                            ActionInput::Button { click } => &click.output,
                            ActionInput::Trigger { pull } => &pull.output,
                            other => {
                                error!("Expected button or trigger path for trigger action for {path} in {action_set_name}, got: {other:?}");
                                continue;
                            }
                        };
                        let action = find_action!(output);
                        let binding = match &action.action {
                            Bool(b) => xr::Binding::new(b, xr_path()),
                            Vector1 { action, .. } => xr::Binding::new(action, xr_path()),
                            _ => panic!("Trigger action should be a bool or float"),
                        };
                        trace!("suggesting {translated} for {}", output);
                        xr_bindings.push(binding);
                    }
                    ActionMode::Dpad | ActionMode::Joystick => {} // TODO
                    ActionMode::Trackpad => {
                        let ActionInput::Vector2 { position } = inputs else {
                            error!("expected vector2 input for {path} in {action_set_name}, got {inputs:?}");
                            continue;
                        };

                        let action = find_action!(position.output);
                        let binding = match &action.action {
                            Vector2 { action, .. } => xr::Binding::new(action, xr_path()),
                            _ => {
                                panic!("Expected Vector2 action for {}", position.output);
                            }
                        };
                        trace!("suggesting {translated} for {}", position.output);
                        xr_bindings.push(binding);
                    }
                    ActionMode::Unknown(mode) => {
                        warn!("unhandled action mode: {mode:?}");
                        continue;
                    }
                }
            }
        }

        let stp = |s| self.openxr.instance.string_to_path(s).unwrap();
        let legacy_bindings = P::legacy_bindings(stp, &legacy_actions);
        let path = stp(P::PROFILE_PATH);
        xr_bindings.extend_from_slice(&legacy_bindings);
        self.openxr
            .instance
            .suggest_interaction_profile_bindings(path, &xr_bindings)
            .unwrap();
    }
}

pub(super) struct PathTranslation {
    pub from: &'static str,
    pub to: &'static str,
}

pub(super) trait InteractionProfile {
    const PROFILE_PATH: &'static str;
    const TRANSLATE_MAP: &'static [PathTranslation];

    fn legal_paths() -> Box<[String]>;
    fn legacy_bindings<'a>(
        string_to_path: impl Fn(&'a str) -> xr::Path,
        actions: &super::LegacyActions,
    ) -> Vec<xr::Binding>;
}
