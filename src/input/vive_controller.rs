use super::{
    action_manifest::{InteractionProfile, PathTranslation},
    LegacyActions,
};
use openxr as xr;
use std::ffi::CStr;

pub struct ViveWands;

impl InteractionProfile for ViveWands {
    const OPENVR_CONTROLLER_TYPE: &'static CStr = c"vive_controller";
    const PROFILE_PATH: &'static str = "/interaction_profiles/htc/vive_controller";
    const TRANSLATE_MAP: &'static [PathTranslation] = &[
        PathTranslation {
            from: "pose/raw",
            to: "input/grip/pose",
        },
        PathTranslation {
            from: "input/grip",
            to: "input/squeeze/click",
        },
        PathTranslation {
            from: "trigger",
            to: "trigger/value",
        },
        PathTranslation {
            from: "application_menu",
            to: "menu/click",
        },
    ];

    fn legal_paths() -> Box<[String]> {
        [
            "input/squeeze/click",
            "input/menu/click",
            "input/trigger/click",
            "input/trigger/value",
            "input/trackpad",
            "input/trackpad/x",
            "input/trackpad/y",
            "input/trackpad/click",
            "input/trackpad/touch",
            "input/grip/pose",
            "input/aim/pose",
            "output/haptic",
        ]
        .iter()
        .flat_map(|s| {
            [
                format!("/user/hand/left/{s}"),
                format!("/user/hand/right/{s}"),
            ]
        })
        .collect()
    }

    fn legacy_bindings<'a>(
        stp: impl Fn(&'a str) -> openxr::Path,
        actions: &LegacyActions,
    ) -> Vec<openxr::Binding> {
        let mut ret = Vec::new();
        macro_rules! both {
            ($action:expr, $path:expr) => {
                ret.push(xr::Binding::new(
                    $action,
                    stp(concat!("/user/hand/left/", $path)),
                ));
                ret.push(xr::Binding::new(
                    $action,
                    stp(concat!("/user/hand/right/", $path)),
                ));
            };
        }

        both!(&actions.pose, "input/grip/pose");
        both!(&actions.trigger, "input/trigger/value");
        both!(&actions.trigger_click, "input/trigger/click");
        both!(&actions.app_menu, "input/menu/click");

        ret
    }
}
