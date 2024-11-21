use super::{
    action_manifest::{InteractionProfile, PathTranslation},
    LegacyActions,
};
use openxr as xr;
use std::ffi::CStr;

pub struct ViveWands;

impl InteractionProfile for ViveWands {
    const OPENVR_CONTROLLER_TYPE: &'static CStr = c"vive_controller";
    const MODEL: &'static CStr = c"Vive. Controller MV";
    const PROFILE_PATH: &'static str = "/interaction_profiles/htc/vive_controller";
    const TRANSLATE_MAP: &'static [PathTranslation] = &[
        PathTranslation {
            from: "pose/raw",
            to: "input/grip/pose",
            stop: true,
        },
        PathTranslation {
            from: "grip",
            to: "squeeze",
            stop: true,
        },
        PathTranslation {
            from: "trigger/pull",
            to: "trigger/value",
            stop: true,
        },
        PathTranslation {
            from: "trigger/click",
            to: "trigger/value",
            stop: true,
        },
        PathTranslation {
            from: "application_menu",
            to: "menu",
            stop: true,
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

    fn legacy_bindings(
        stp: impl for<'a> Fn(&'a str) -> openxr::Path,
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

        both!(&actions.grip_pose, "input/grip/pose");
        both!(&actions.aim_pose, "input/aim/pose");
        both!(&actions.trigger, "input/trigger/value");
        both!(&actions.trigger_click, "input/trigger/click");
        both!(&actions.app_menu, "input/menu/click");
        both!(&actions.squeeze, "input/squeeze/click");

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::{xr, InteractionProfile, ViveWands};
    use crate::input::tests::Fixture;

    #[test]
    fn verify_bindings() {
        let f = Fixture::new();
        f.load_actions(c"actions.json");
        f.verify_bindings::<bool>(
            ViveWands::PROFILE_PATH,
            c"/actions/set1/in/boolact",
            [
                "/user/hand/left/input/squeeze/click".into(),
                "/user/hand/right/input/squeeze/click".into(),
                "/user/hand/left/input/menu/click".into(),
                "/user/hand/right/input/menu/click".into(),
                // Suggesting float paths for boolean inputs is legal
                "/user/hand/left/input/trigger/value".into(),
                "/user/hand/right/input/trigger/value".into(),
                "/user/hand/left/input/trackpad/click".into(),
                "/user/hand/left/input/trackpad/touch".into(),
            ],
        );

        f.verify_bindings::<f32>(
            ViveWands::PROFILE_PATH,
            c"/actions/set1/in/vec1act",
            [
                "/user/hand/left/input/trigger/value".into(),
                "/user/hand/right/input/trigger/value".into(),
                "/user/hand/right/input/squeeze/click".into(),
            ],
        );

        f.verify_bindings::<xr::Vector2f>(
            ViveWands::PROFILE_PATH,
            c"/actions/set1/in/vec2act",
            [
                "/user/hand/left/input/trackpad".into(),
                "/user/hand/right/input/trackpad".into(),
            ],
        );

        f.verify_bindings::<xr::Haptic>(
            ViveWands::PROFILE_PATH,
            c"/actions/set1/in/vib",
            [
                "/user/hand/left/output/haptic".into(),
                "/user/hand/right/output/haptic".into(),
            ],
        );
    }
}
