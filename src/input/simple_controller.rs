use super::{
    action_manifest::{InteractionProfile, PathTranslation, StringToPath},
    legacy::LegacyBindings,
};
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

    fn legacy_bindings(stp: impl StringToPath) -> LegacyBindings {
        LegacyBindings {
            grip_pose: stp.leftright("input/grip/pose"),
            aim_pose: stp.leftright("input/aim/pose"),
            trigger: stp.leftright("input/select/click"),
            trigger_click: stp.leftright("input/select/click"),
            app_menu: stp.leftright("input/menu/click"),
            squeeze: stp.leftright("input/menu/click"),
        }
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
