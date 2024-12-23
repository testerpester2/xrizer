use crate::openxr_data::SessionData;
use log::{debug, trace};
use openxr as xr;
use std::sync::OnceLock;

macro_rules! legacy_actions_and_bindings {
    ($($field:ident: $ty:ty),+$(,)?) => {
        pub(super) struct LegacyActions {
            $(pub $field: $ty),+
        }
        pub(super) struct LegacyBindings {
            $(pub $field: Vec<xr::Path>),+
        }
        impl LegacyBindings {
            pub fn binding_iter(self, actions: &LegacyActions) -> impl Iterator<Item = xr::Binding<'_>> {
                std::iter::empty()
                $(
                    .chain(
                        self.$field.into_iter().map(|binding| xr::Binding::new(&actions.$field, binding))
                    )
                )+
            }
        }
    }
}

legacy_actions_and_bindings! {
    grip_pose: xr::Action<xr::Posef>,
    aim_pose: xr::Action<xr::Posef>,
    app_menu: xr::Action<bool>,
    trigger_click: xr::Action<bool>,
    trigger: xr::Action<f32>,
    squeeze: xr::Action<f32>,
}

pub(super) struct LegacyActionData {
    pub set: xr::ActionSet,
    pub left_spaces: HandSpaces,
    pub right_spaces: HandSpaces,
    pub actions: LegacyActions,
}

impl LegacyActionData {
    pub fn new<'a>(
        instance: &'a xr::Instance,
        session: &'a xr::Session<xr::vulkan::Vulkan>,
        left_hand: xr::Path,
        right_hand: xr::Path,
    ) -> Self {
        debug!("creating legacy actions");
        let leftright = [left_hand, right_hand];
        let set = instance
            .create_action_set("xrizer-legacy-set", "XRizer Legacy Set", 0)
            .unwrap();
        let grip_pose = set
            .create_action("grip-pose", "Grip Pose", &leftright)
            .unwrap();
        let aim_pose = set
            .create_action("aim-pose", "Aim Pose", &leftright)
            .unwrap();
        let trigger_click = set
            .create_action("trigger-click", "Trigger Click", &leftright)
            .unwrap();
        let trigger = set.create_action("trigger", "Trigger", &leftright).unwrap();
        let squeeze = set.create_action("squeeze", "Squeeze", &leftright).unwrap();
        let app_menu = set
            .create_action("app-menu", "Application Menu", &leftright)
            .unwrap();

        let create_spaces = |hand| HandSpaces {
            hand_path: hand,
            grip: grip_pose
                .create_space(session, hand, xr::Posef::IDENTITY)
                .unwrap(),
            aim: aim_pose
                .create_space(session, hand, xr::Posef::IDENTITY)
                .unwrap(),
            raw: OnceLock::new(),
        };

        let left_spaces = create_spaces(left_hand);
        let right_spaces = create_spaces(right_hand);

        Self {
            set,
            left_spaces,
            right_spaces,
            actions: LegacyActions {
                grip_pose,
                aim_pose,
                app_menu,
                trigger_click,
                trigger,
                squeeze,
            },
        }
    }
}

pub(super) struct HandSpaces {
    hand_path: xr::Path,
    grip: xr::Space,
    aim: xr::Space,

    /// Based on the controller jsons in SteamVR, the "raw" pose
    /// (which seems to be equivalent to the pose returned by WaitGetPoses)
    /// is actually the grip pose, but in the same position as the aim pose.
    /// Using this pose instead of the grip fixes strange controller rotation in
    /// I Expect You To Die 3.
    /// This is stored as a space so we can locate hand joints relative to it for skeletal data.
    raw: OnceLock<xr::Space>,
}

impl HandSpaces {
    pub fn try_get_or_init_raw(
        &self,
        data: &SessionData,
        actions: &LegacyActions,
        time: xr::Time,
    ) -> Option<&xr::Space> {
        if let Some(raw) = self.raw.get() {
            return Some(raw);
        }

        // This offset between grip and aim poses should be static,
        // so it should be fine to only grab it once.
        let aim_loc = self.aim.locate(&self.grip, time).unwrap();
        if !aim_loc.location_flags.contains(
            xr::SpaceLocationFlags::POSITION_VALID | xr::SpaceLocationFlags::ORIENTATION_VALID,
        ) {
            trace!("couldn't locate aim pose, no raw space will be created");
            return None;
        }

        self.raw
            .set(
                actions
                    .grip_pose
                    .create_space(
                        &data.session,
                        self.hand_path,
                        xr::Posef {
                            orientation: xr::Quaternionf::IDENTITY,
                            position: aim_loc.pose.position,
                        },
                    )
                    .unwrap(),
            )
            .unwrap_or_else(|_| unreachable!());

        self.raw.get()
    }
}
