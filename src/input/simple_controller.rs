use super::{
    action_manifest::{InteractionProfile, PathTranslation},
    LegacyActions,
};
use openxr as xr;
use std::ffi::CStr;

pub struct SimpleController;

impl InteractionProfile for SimpleController {
    const OPENVR_CONTROLLER_TYPE: &'static CStr = c"generic"; // meaningless really
    const MODEL: &'static CStr = c"<unknown>";
    const PROFILE_PATH: &'static str = "/interaction_profiles/khr/simple_controller";
    const TRANSLATE_MAP: &'static [PathTranslation] = &[
        PathTranslation {
            from: "trigger",
            to: "select",
            stop: true,
        },
        PathTranslation {
            from: "application_menu",
            to: "menu",
            stop: true,
        },
    ];

    fn legacy_bindings(
        string_to_path: impl for<'a> Fn(&'a str) -> openxr::Path,
        actions: &LegacyActions,
    ) -> Vec<openxr::Binding> {
        let mut ret = Vec::new();
        let stp = string_to_path;
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

        both!(&actions.grip_pose, "input/grip/pose");
        both!(&actions.aim_pose, "input/aim/pose");
        both!(&actions.trigger, "input/select/click");
        both!(&actions.trigger_click, "input/select/click");
        both!(&actions.app_menu, "input/menu/click");
        both!(&actions.squeeze, "input/menu/click");

        ret
    }

    fn legal_paths() -> Box<[String]> {
        [
            "input/select/click",
            "input/menu/click",
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
}
