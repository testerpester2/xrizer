use super::{action_manifest::PathTranslation, InteractionProfile};
use openxr as xr;

pub struct Knuckles;

impl InteractionProfile for Knuckles {
    const PROFILE_PATH: &'static str = "/interaction_profiles/valve/index_controller";
    const MODEL: &'static std::ffi::CStr = c"Knuckles";
    const OPENVR_CONTROLLER_TYPE: &'static std::ffi::CStr = c"knuckles";
    const TRANSLATE_MAP: &'static [PathTranslation] = &[
        PathTranslation {
            from: "pose/raw",
            to: "input/grip/pose",
            stop: true,
        },
        PathTranslation {
            from: "pose/gdc2015",
            to: "input/grip/pose",
            stop: true,
        },
        PathTranslation {
            from: "pull",
            to: "value",
            stop: true,
        },
        PathTranslation {
            from: "input/grip",
            to: "input/squeeze",
            stop: false,
        },
        PathTranslation {
            from: "squeeze/click",
            to: "squeeze/force",
            stop: true,
        },
        PathTranslation {
            from: "squeeze/grab",
            to: "squeeze/force",
            stop: true
        },
        PathTranslation {
            from: "trackpad/click",
            to: "trackpad/force",
            stop: true,
        },
    ];

    fn legal_paths() -> Box<[String]> {
        let click_and_touch = ["input/a", "input/b", "input/trigger", "input/thumbstick"]
            .iter()
            .flat_map(|p| [format!("{p}/click"), format!("{p}/touch")]);
        let x_and_y = ["input/thumbstick", "input/trackpad"]
            .iter()
            .flat_map(|p| [format!("{p}/x"), format!("{p}/y"), p.to_string()]);
        let misc = [
            "input/squeeze/value",
            "input/squeeze/force",
            "input/trigger/value",
            "input/trackpad/force",
            "input/trackpad/touch",
            "input/grip/pose",
            "input/aim/pose",
            "output/haptic",
        ]
        .into_iter()
        .map(String::from);

        click_and_touch
            .chain(x_and_y)
            .chain(misc)
            .flat_map(|p| {
                [
                    format!("/user/hand/left/{p}"),
                    format!("/user/hand/right/{p}"),
                ]
            })
            .collect()
    }

    fn legacy_bindings(
        string_to_path: impl for<'a> Fn(&'a str) -> openxr::Path,
        actions: &super::LegacyActions,
    ) -> Vec<openxr::Binding> {
        let stp = string_to_path;
        let mut bindings = Vec::new();

        macro_rules! add_binding {
            ($action:expr, $suffix:literal) => {
                bindings.push(xr::Binding::new(
                    $action,
                    stp(concat!("/user/hand/left/", $suffix)),
                ));
                bindings.push(xr::Binding::new(
                    $action,
                    stp(concat!("/user/hand/right/", $suffix)),
                ));
            };
        }

        add_binding!(&actions.pose, "input/grip/pose");
        add_binding!(&actions.app_menu, "input/b/click");
        add_binding!(&actions.trigger, "input/trigger/value");
        add_binding!(&actions.trigger_click, "input/trigger/click");

        bindings
    }
}

#[cfg(test)]
mod tests {
    use super::{xr, InteractionProfile, Knuckles};
    use crate::input::tests::Fixture;

    #[test]
    fn verify_bindings() {
        let f = Fixture::new();
        f.load_actions(c"actions.json");

        let path = Knuckles::PROFILE_PATH;
        f.verify_bindings::<bool>(
            path,
            c"/actions/set1/in/boolact",
            [
                "/user/hand/left/input/a/click".into(),
                "/user/hand/right/input/a/click".into(),
                "/user/hand/left/input/b/click".into(),
                "/user/hand/right/input/b/click".into(),
                "/user/hand/left/input/trigger/click".into(),
                "/user/hand/right/input/trigger/click".into(),
                "/user/hand/left/input/trigger/touch".into(),
                "/user/hand/right/input/trigger/touch".into(),
                "/user/hand/left/input/thumbstick/click".into(),
                "/user/hand/right/input/thumbstick/click".into(),
                "/user/hand/left/input/thumbstick/touch".into(),
                "/user/hand/right/input/thumbstick/touch".into(),
                "/user/hand/right/input/trackpad/touch".into(),
                "/user/hand/left/input/squeeze/force".into(),
                "/user/hand/left/input/trackpad/force".into(),
                "/user/hand/right/input/trackpad/force".into(),
            ],
        );

        f.verify_bindings::<f32>(
            path,
            c"/actions/set1/in/vec1act",
            [
                "/user/hand/left/input/trigger/value".into(),
                "/user/hand/right/input/trigger/value".into(),
                "/user/hand/left/input/squeeze/force".into(),
                "/user/hand/right/input/squeeze/force".into(),
            ],
        );

        f.verify_bindings::<xr::Vector2f>(
            path,
            c"/actions/set1/in/vec2act",
            [
                "/user/hand/left/input/trackpad".into(),
                "/user/hand/right/input/trackpad".into(),
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

        f.verify_bindings::<xr::Posef>(
            path,
            c"/actions/set1/in/pose",
            [
                "/user/hand/left/input/grip/pose".into(),
                "/user/hand/right/input/grip/pose".into(),
            ],
        );
    }
}
