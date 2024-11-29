use super::{
    action_manifest::{InteractionProfile, PathTranslation},
    LegacyActions,
};
use openxr as xr;
use std::ffi::CStr;

pub struct Touch;

impl InteractionProfile for Touch {
    const OPENVR_CONTROLLER_TYPE: &'static CStr = c"oculus_touch";
    const MODEL: &'static CStr = c"Miramar";
    const PROFILE_PATH: &'static str = "/interaction_profiles/oculus/touch_controller";
    const TRANSLATE_MAP: &'static [PathTranslation] = &[
        PathTranslation {
            from: "trigger/click",
            to: "trigger/value",
            stop: true,
        },
        PathTranslation {
            from: "grip/click",
            to: "squeeze/value",
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
        PathTranslation {
            from: "joystick",
            to: "thumbstick",
            stop: true,
        },
    ];

    fn legacy_bindings(
        string_to_path: impl for<'a> Fn(&'a str) -> openxr::Path,
        actions: &LegacyActions,
    ) -> Vec<openxr::Binding> {
        let mut bindings = Vec::new();

        macro_rules! add_binding {
            ($action:expr, $suffix:literal) => {
                bindings.push(xr::Binding::new(
                    $action,
                    string_to_path(concat!("/user/hand/left/", $suffix)),
                ));
                bindings.push(xr::Binding::new(
                    $action,
                    string_to_path(concat!("/user/hand/right/", $suffix)),
                ));
            };
        }

        bindings.push(xr::Binding::new(
            &actions.app_menu,
            string_to_path("/user/hand/left/input/menu/click"),
        ));
        add_binding!(&actions.grip_pose, "input/grip/pose");
        add_binding!(&actions.aim_pose, "input/aim/pose");
        add_binding!(&actions.trigger, "input/trigger/value");
        add_binding!(&actions.trigger_click, "input/trigger/value");
        add_binding!(&actions.squeeze, "input/squeeze/value");

        bindings
    }

    fn legal_paths() -> Box<[String]> {
        let left_only = [
            "input/x/click",
            "input/x/touch",
            "input/y/click",
            "input/y/touch",
            "input/menu/click",
        ]
        .iter()
        .map(|p| format!("/user/hand/left/{p}"));
        let right_only = [
            "input/a/click",
            "input/a/touch",
            "input/b/click",
            "input/b/touch",
        ]
        .iter()
        .map(|p| format!("/user/hand/right/{p}"));

        let both = [
            "input/squeeze/value",
            "input/trigger/value",
            "input/trigger/touch",
            "input/thumbstick",
            "input/thumbstick/x",
            "input/thumbstick/y",
            "input/thumbstick/click",
            "input/thumbstick/touch",
            "input/thumbrest/touch",
            "input/grip/pose",
            "input/aim/pose",
            "output/haptic",
        ]
        .iter()
        .flat_map(|p| {
            [
                format!("/user/hand/left/{p}"),
                format!("/user/hand/right/{p}"),
            ]
        });

        left_only.chain(right_only).chain(both).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{xr, InteractionProfile, Touch};
    use crate::input::tests::Fixture;

    #[test]
    fn verify_bindings() {
        let f = Fixture::new();
        f.load_actions(c"actions.json");

        let path = Touch::PROFILE_PATH;
        f.verify_bindings::<bool>(
            path,
            c"/actions/set1/in/boolact",
            [
                "/user/hand/left/input/x/click".into(),
                "/user/hand/left/input/y/click".into(),
                "/user/hand/right/input/a/click".into(),
                "/user/hand/right/input/b/click".into(),
                "/user/hand/left/input/squeeze/value".into(),
                "/user/hand/right/input/squeeze/value".into(),
                "/user/hand/right/input/thumbstick/click".into(),
                "/user/hand/right/input/thumbstick/touch".into(),
                "/user/hand/left/input/menu/click".into(),
                "/user/hand/left/input/trigger/value".into(),
                "/user/hand/right/input/trigger/value".into(),
            ],
        );

        f.verify_bindings::<f32>(
            path,
            c"/actions/set1/in/vec1act",
            [
                "/user/hand/left/input/trigger/value".into(),
                "/user/hand/right/input/trigger/value".into(),
            ],
        );

        f.verify_bindings::<xr::Vector2f>(
            path,
            c"/actions/set1/in/vec2act",
            [
                "/user/hand/left/input/thumbstick".into(),
                "/user/hand/right/input/thumbstick".into(),
            ],
        );

        f.verify_bindings::<xr::Haptic>(
            path,
            c"/actions/set1/in/vib",
            [
                "/user/hand/left/output/haptic".into(),
                "/user/hand/right/output/haptic".into(),
            ],
        );
    }
}
