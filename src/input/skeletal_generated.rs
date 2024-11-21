use super::HandSkeletonBone;
use crate::vr;

// The bone data in this file is given in parent space. Using parent space
// allows for lerping between poses to always work correctly, and the bones can be
// easily transformed to model space when needed.
pub mod left_hand {
    use super::*;
    pub static BINDPOSE: [vr::VRBoneTransform_t; HandSkeletonBone::Count as usize] = [
        // Root
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00000, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.00000,
                x: -0.00000,
                y: -1.00000,
                z: -0.00000,
            },
        },
        // Wrist
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00016, -0.00003, -0.00063, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // Thumb1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01791, 0.02918, 0.02530, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.27639,
                x: 0.54119,
                y: 0.18203,
                z: 0.77304,
            },
        },
        // Thumb2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04041, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.96917,
                x: 0.00006,
                y: -0.00137,
                z: 0.24638,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03252, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98817,
                x: 0.00010,
                y: 0.00140,
                z: 0.15334,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03046, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // IndexFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00156, 0.02107, 0.01479, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.55075,
                x: 0.53106,
                y: -0.35143,
                z: 0.53958,
            },
        },
        // IndexFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.07380, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.96898,
                x: 0.00162,
                y: -0.05289,
                z: 0.24140,
            },
        },
        // IndexFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04329, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98277,
                x: -0.00009,
                y: 0.00504,
                z: 0.18476,
            },
        },
        // IndexFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02828, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99707,
                x: 0.00003,
                y: -0.00117,
                z: 0.07646,
            },
        },
        // IndexFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02282, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // MiddleFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00218, 0.00712, 0.01632, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.53342,
                x: 0.56175,
                y: -0.41974,
                z: 0.47299,
            },
        },
        // MiddleFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.07089, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.97339,
                x: 0.00000,
                y: -0.00019,
                z: 0.22916,
            },
        },
        // MiddleFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04311, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98753,
                x: 0.00009,
                y: -0.00369,
                z: 0.15740,
            },
        },
        // MiddleFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03327, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98996,
                x: -0.00011,
                y: 0.00413,
                z: 0.14128,
            },
        },
        // MiddleFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02589, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // RingFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00051, -0.00655, 0.01635, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.51669,
                x: 0.55014,
                y: -0.49555,
                z: 0.42989,
            },
        },
        // RingFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.06597, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.97456,
                x: -0.00090,
                y: -0.04096,
                z: 0.22037,
            },
        },
        // RingFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04033, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99100,
                x: -0.00007,
                y: 0.00253,
                z: 0.13383,
            },
        },
        // RingFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02849, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99079,
                x: 0.00020,
                y: -0.00426,
                z: 0.13535,
            },
        },
        // RingFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02243, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // PinkyFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00248, -0.01898, 0.01521, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.48576,
                x: 0.51533,
                y: -0.61502,
                z: 0.34675,
            },
        },
        // PinkyFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.06286, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99349,
                x: 0.00394,
                y: 0.02816,
                z: 0.11031,
            },
        },
        // PinkyFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02987, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99111,
                x: 0.00038,
                y: -0.01146,
                z: 0.13252,
            },
        },
        // PinkyFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01798, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99401,
                x: -0.00054,
                y: 0.01270,
                z: 0.10858,
            },
        },
        // PinkyFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01802, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // Aux_Thumb
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03928, 0.06008, 0.08449, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.04861,
                x: -0.56911,
                y: 0.04504,
                z: -0.81959,
            },
        },
        // Aux_IndexFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01823, 0.03728, 0.14896, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.20956,
                x: 0.31233,
                y: -0.59723,
                z: 0.70842,
            },
        },
        // Aux_MiddleFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01256, 0.00787, 0.15469, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.22114,
                x: 0.27117,
                y: -0.64706,
                z: 0.67740,
            },
        },
        // Aux_RingFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01787, -0.02324, 0.14224, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.23741,
                x: 0.26235,
                y: -0.72163,
                z: 0.59503,
            },
        },
        // Aux_PinkyFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01601, -0.04565, 0.11928, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.34900,
                x: 0.26548,
                y: -0.73903,
                z: 0.51142,
            },
        },
    ];
    pub static OPENHAND: [vr::VRBoneTransform_t; HandSkeletonBone::Count as usize] = [
        // Root
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00000, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.00000,
                x: -0.00000,
                y: -1.00000,
                z: -0.00000,
            },
        },
        // Wrist
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00016, -0.00003, -0.00063, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // Thumb1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01791, 0.02918, 0.02530, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.43792,
                x: 0.56781,
                y: 0.11983,
                z: 0.68663,
            },
        },
        // Thumb2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04041, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99031,
                x: 0.04887,
                y: 0.05609,
                z: 0.11728,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03252, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99493,
                x: 0.08159,
                y: 0.04521,
                z: -0.03764,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03046, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // IndexFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00156, 0.02107, 0.01479, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.55075,
                x: 0.53106,
                y: -0.35143,
                z: 0.53958,
            },
        },
        // IndexFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.07380, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99318,
                x: 0.06183,
                y: 0.04100,
                z: 0.08997,
            },
        },
        // IndexFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04329, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98958,
                x: -0.14077,
                y: -0.01481,
                z: -0.02620,
            },
        },
        // IndexFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02828, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99707,
                x: 0.00003,
                y: -0.00117,
                z: 0.07646,
            },
        },
        // IndexFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02282, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // MiddleFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00218, 0.00712, 0.01632, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.53342,
                x: 0.56175,
                y: -0.41974,
                z: 0.47299,
            },
        },
        // MiddleFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.07089, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99087,
                x: -0.03929,
                y: -0.01545,
                z: 0.12805,
            },
        },
        // MiddleFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04311, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99882,
                x: -0.04623,
                y: -0.01254,
                z: -0.00778,
            },
        },
        // MiddleFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03327, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99920,
                x: -0.03577,
                y: 0.00817,
                z: 0.01562,
            },
        },
        // MiddleFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02589, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // RingFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00051, -0.00655, 0.01635, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.51669,
                x: 0.55014,
                y: -0.49555,
                z: 0.42989,
            },
        },
        // RingFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.06597, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98958,
                x: -0.04109,
                y: -0.08942,
                z: 0.10511,
            },
        },
        // RingFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04033, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99475,
                x: -0.07026,
                y: 0.04928,
                z: -0.05571,
            },
        },
        // RingFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02849, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99079,
                x: 0.00020,
                y: -0.00426,
                z: 0.13535,
            },
        },
        // RingFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02243, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // PinkyFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00248, -0.01898, 0.01521, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.48576,
                x: 0.51533,
                y: -0.61502,
                z: 0.34675,
            },
        },
        // PinkyFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.06286, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99505,
                x: 0.00921,
                y: -0.09085,
                z: 0.03929,
            },
        },
        // PinkyFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02987, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99406,
                x: -0.09116,
                y: -0.01704,
                z: -0.05688,
            },
        },
        // PinkyFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01798, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99768,
                x: -0.02291,
                y: 0.01686,
                z: 0.06191,
            },
        },
        // PinkyFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01802, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // Aux_Thumb
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02218, 0.07865, 0.07720, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.29496,
                x: 0.54402,
                y: 0.20688,
                z: 0.75779,
            },
        },
        // Aux_IndexFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01112, 0.05525, 0.15428, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.51341,
                x: 0.42038,
                y: -0.45372,
                z: 0.59483,
            },
        },
        // Aux_MiddleFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01291, 0.00792, 0.16137, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.51675,
                x: 0.44600,
                y: -0.55996,
                z: 0.46957,
            },
        },
        // Aux_RingFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00763, -0.02903, 0.14747, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.47395,
                x: 0.40717,
                y: -0.65043,
                z: 0.43188,
            },
        },
        // Aux_PinkyFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00586, -0.05994, 0.11671, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.46974,
                x: 0.47051,
                y: -0.70428,
                z: 0.24892,
            },
        },
    ];
    pub static SQUEEZE: [vr::VRBoneTransform_t; HandSkeletonBone::Count as usize] = [
        // Root
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00000, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.00000,
                x: -0.00000,
                y: -1.00000,
                z: -0.00000,
            },
        },
        // Wrist
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00016, -0.00003, -0.00063, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // Thumb1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01791, 0.02918, 0.02530, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.27639,
                x: 0.54119,
                y: 0.18203,
                z: 0.77304,
            },
        },
        // Thumb2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04041, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99031,
                x: 0.04887,
                y: 0.05609,
                z: 0.11728,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03252, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98817,
                x: 0.00010,
                y: 0.00140,
                z: 0.15334,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03046, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // IndexFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00139, 0.01484, 0.01476, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.50360,
                x: 0.55558,
                y: -0.33906,
                z: 0.56812,
            },
        },
        // IndexFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.07380, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.68856,
                x: -0.10477,
                y: 0.02622,
                z: 0.71709,
            },
        },
        // IndexFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04329, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.77070,
                x: -0.10731,
                y: -0.07406,
                z: 0.62371,
            },
        },
        // IndexFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02828, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.72833,
                x: 0.11408,
                y: 0.06229,
                z: 0.67279,
            },
        },
        // IndexFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02282, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: -0.00000,
                z: -0.00000,
            },
        },
        // MiddleFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00239, 0.00507, 0.01631, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.47825,
                x: 0.57711,
                y: -0.38315,
                z: 0.53983,
            },
        },
        // MiddleFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.07089, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.70251,
                x: 0.00968,
                y: 0.08009,
                z: 0.70709,
            },
        },
        // MiddleFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04311, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.72563,
                x: -0.09510,
                y: 0.03877,
                z: 0.68038,
            },
        },
        // MiddleFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03327, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.80974,
                x: 0.07168,
                y: 0.01950,
                z: 0.58207,
            },
        },
        // MiddleFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02589, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // RingFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00240, -0.00604, 0.01627, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.51773,
                x: 0.53846,
                y: -0.48437,
                z: 0.45541,
            },
        },
        // RingFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.06597, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.74485,
                x: 0.04915,
                y: -0.06633,
                z: 0.66210,
            },
        },
        // RingFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.04033, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.71214,
                x: 0.03370,
                y: -0.01893,
                z: 0.70097,
            },
        },
        // RingFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02849, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.81247,
                x: 0.01195,
                y: 0.00995,
                z: 0.58280,
            },
        },
        // RingFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02243, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // PinkyFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00370, -0.01332, 0.01647, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.50294,
                x: 0.52823,
                y: -0.54218,
                z: 0.41721,
            },
        },
        // PinkyFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.06286, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.72357,
                x: 0.25616,
                y: 0.01231,
                z: 0.64083,
            },
        },
        // PinkyFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02987, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.75307,
                x: -0.05787,
                y: 0.02975,
                z: 0.65472,
            },
        },
        // PinkyFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01798, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.74162,
                x: -0.02136,
                y: 0.00875,
                z: 0.67042,
            },
        },
        // PinkyFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01802, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: 0.00000,
            },
        },
        // Aux_Thumb
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04135, 0.06801, 0.08090, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.01872,
                x: 0.54615,
                y: 0.08747,
                z: 0.83290,
            },
        },
        // Aux_IndexFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04379, 0.02325, 0.06410, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.61286,
                x: 0.75056,
                y: 0.23429,
                z: -0.07859,
            },
        },
        // Aux_MiddleFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03729, 0.00292, 0.05840, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.65101,
                x: 0.70942,
                y: 0.23326,
                z: -0.13605,
            },
        },
        // Aux_RingFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03532, -0.01631, 0.06226, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.67249,
                x: 0.68173,
                y: 0.24639,
                z: -0.14932,
            },
        },
        // Aux_PinkyFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02603, -0.03303, 0.06707, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.72597,
                x: 0.63292,
                y: 0.26608,
                z: -0.03987,
            },
        },
    ];
}

pub mod right_hand {
    use super::*;
    pub static BINDPOSE: [vr::VRBoneTransform_t; HandSkeletonBone::Count as usize] = [
        // Root
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00000, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.00000,
                x: -0.00000,
                y: -1.00000,
                z: -0.00000,
            },
        },
        // Wrist
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00016, -0.00003, -0.00063, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // Thumb1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01791, 0.02918, 0.02530, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.54119,
                x: -0.27639,
                y: 0.77304,
                z: -0.18203,
            },
        },
        // Thumb2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04041, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.96917,
                x: 0.00006,
                y: -0.00137,
                z: 0.24638,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03252, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98817,
                x: 0.00010,
                y: 0.00140,
                z: 0.15334,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03046, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // IndexFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00156, 0.02107, 0.01479, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.53106,
                x: -0.55075,
                y: 0.53958,
                z: 0.35143,
            },
        },
        // IndexFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.07380, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.96898,
                x: 0.00162,
                y: -0.05289,
                z: 0.24140,
            },
        },
        // IndexFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04329, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98277,
                x: -0.00009,
                y: 0.00504,
                z: 0.18476,
            },
        },
        // IndexFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02828, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99707,
                x: 0.00003,
                y: -0.00117,
                z: 0.07646,
            },
        },
        // IndexFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02282, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // MiddleFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00218, 0.00712, 0.01632, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.56175,
                x: -0.53342,
                y: 0.47299,
                z: 0.41974,
            },
        },
        // MiddleFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.07089, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.97339,
                x: 0.00000,
                y: -0.00019,
                z: 0.22916,
            },
        },
        // MiddleFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04311, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98753,
                x: 0.00009,
                y: -0.00369,
                z: 0.15740,
            },
        },
        // MiddleFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03327, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98996,
                x: -0.00011,
                y: 0.00413,
                z: 0.14128,
            },
        },
        // MiddleFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02589, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // RingFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00051, -0.00655, 0.01635, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.55014,
                x: -0.51669,
                y: 0.42989,
                z: 0.49555,
            },
        },
        // RingFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.06597, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.97456,
                x: -0.00090,
                y: -0.04096,
                z: 0.22037,
            },
        },
        // RingFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04033, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99100,
                x: -0.00007,
                y: 0.00253,
                z: 0.13383,
            },
        },
        // RingFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02849, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99079,
                x: 0.00020,
                y: -0.00426,
                z: 0.13535,
            },
        },
        // RingFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02243, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // PinkyFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00248, -0.01898, 0.01521, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.51533,
                x: -0.48576,
                y: 0.34675,
                z: 0.61502,
            },
        },
        // PinkyFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.06286, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99349,
                x: 0.00394,
                y: 0.02816,
                z: 0.11031,
            },
        },
        // PinkyFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02987, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99111,
                x: 0.00038,
                y: -0.01146,
                z: 0.13252,
            },
        },
        // PinkyFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01798, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99401,
                x: -0.00054,
                y: 0.01270,
                z: 0.10858,
            },
        },
        // PinkyFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01802, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // Aux_Thumb
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03928, 0.06008, 0.08449, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.56911,
                x: 0.04861,
                y: 0.81959,
                z: 0.04504,
            },
        },
        // Aux_IndexFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01823, 0.03728, 0.14896, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.31233,
                x: -0.20956,
                y: 0.70842,
                z: 0.59723,
            },
        },
        // Aux_MiddleFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01256, 0.00787, 0.15469, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.27117,
                x: -0.22114,
                y: 0.67740,
                z: 0.64706,
            },
        },
        // Aux_RingFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01787, -0.02324, 0.14224, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.26235,
                x: -0.23741,
                y: 0.59503,
                z: 0.72163,
            },
        },
        // Aux_PinkyFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01601, -0.04565, 0.11928, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.26548,
                x: -0.34900,
                y: 0.51142,
                z: 0.73903,
            },
        },
    ];
    pub static OPENHAND: [vr::VRBoneTransform_t; HandSkeletonBone::Count as usize] = [
        // Root
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00000, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.00000,
                x: -0.00000,
                y: 1.00000,
                z: 0.00000,
            },
        },
        // Wrist
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00016, -0.00003, -0.00063, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // Thumb1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01791, 0.02918, 0.02530, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.56781,
                x: -0.43792,
                y: 0.68663,
                z: -0.11983,
            },
        },
        // Thumb2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04041, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99031,
                x: 0.04887,
                y: 0.05609,
                z: 0.11728,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03252, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99493,
                x: 0.08159,
                y: 0.04521,
                z: -0.03764,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03046, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // IndexFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00156, 0.02107, 0.01479, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.53106,
                x: -0.55075,
                y: 0.53958,
                z: 0.35143,
            },
        },
        // IndexFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.07380, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99318,
                x: 0.06183,
                y: 0.04100,
                z: 0.08997,
            },
        },
        // IndexFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04329, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98958,
                x: -0.14077,
                y: -0.01481,
                z: -0.02620,
            },
        },
        // IndexFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02828, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99707,
                x: 0.00003,
                y: -0.00117,
                z: 0.07646,
            },
        },
        // IndexFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02282, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // MiddleFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00218, 0.00712, 0.01632, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.56175,
                x: -0.53342,
                y: 0.47299,
                z: 0.41974,
            },
        },
        // MiddleFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.07089, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99087,
                x: -0.03929,
                y: -0.01545,
                z: 0.12805,
            },
        },
        // MiddleFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04311, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99882,
                x: -0.04623,
                y: -0.01254,
                z: -0.00778,
            },
        },
        // MiddleFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03327, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99920,
                x: -0.03577,
                y: 0.00817,
                z: 0.01562,
            },
        },
        // MiddleFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02589, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // RingFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00051, -0.00655, 0.01635, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.55014,
                x: -0.51669,
                y: 0.42989,
                z: 0.49555,
            },
        },
        // RingFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.06597, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98958,
                x: -0.04109,
                y: -0.08942,
                z: 0.10511,
            },
        },
        // RingFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04033, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99475,
                x: -0.07026,
                y: 0.04928,
                z: -0.05571,
            },
        },
        // RingFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02849, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99079,
                x: 0.00020,
                y: -0.00426,
                z: 0.13535,
            },
        },
        // RingFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02243, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // PinkyFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00248, -0.01898, 0.01521, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.51533,
                x: -0.48576,
                y: 0.34675,
                z: 0.61502,
            },
        },
        // PinkyFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.06286, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99505,
                x: 0.00921,
                y: -0.09085,
                z: 0.03929,
            },
        },
        // PinkyFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02987, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99406,
                x: -0.09116,
                y: -0.01704,
                z: -0.05688,
            },
        },
        // PinkyFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01798, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.99768,
                x: -0.02291,
                y: 0.01686,
                z: 0.06191,
            },
        },
        // PinkyFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01802, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // Aux_Thumb
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02218, 0.07865, 0.07720, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.54403,
                x: -0.29496,
                y: 0.75779,
                z: -0.20688,
            },
        },
        // Aux_IndexFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01112, 0.05525, 0.15428, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.42038,
                x: -0.51341,
                y: 0.59483,
                z: 0.45372,
            },
        },
        // Aux_MiddleFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01291, 0.00792, 0.16137, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.44600,
                x: -0.51675,
                y: 0.46957,
                z: 0.55996,
            },
        },
        // Aux_RingFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00763, -0.02903, 0.14747, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.40717,
                x: -0.47395,
                y: 0.43188,
                z: 0.65043,
            },
        },
        // Aux_PinkyFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00586, -0.05994, 0.11671, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.47050,
                x: -0.46974,
                y: 0.24892,
                z: 0.70428,
            },
        },
    ];
    pub static SQUEEZE: [vr::VRBoneTransform_t; HandSkeletonBone::Count as usize] = [
        // Root
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00000, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.00000,
                x: -0.00000,
                y: -1.00000,
                z: -0.00000,
            },
        },
        // Wrist
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00016, -0.00003, -0.00063, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // Thumb1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.01791, 0.02918, 0.02530, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.54119,
                x: -0.27639,
                y: 0.77304,
                z: -0.18203,
            },
        },
        // Thumb2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04041, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.96917,
                x: 0.00006,
                y: -0.00137,
                z: 0.24638,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03252, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.98817,
                x: 0.00010,
                y: 0.00140,
                z: 0.15334,
            },
        },
        // Thumb3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03046, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // IndexFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.00218, 0.01773, 0.01625, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.57080,
                x: -0.51658,
                y: 0.53668,
                z: 0.34542,
            },
        },
        // IndexFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.07407, -0.00037, -0.00460, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.64992,
                x: -0.14599,
                y: 0.06483,
                z: 0.74302,
            },
        },
        // IndexFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04329, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.76851,
                x: -0.05789,
                y: -0.01468,
                z: 0.63705,
            },
        },
        // IndexFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02828, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.81120,
                x: -0.06082,
                y: -0.13770,
                z: 0.56505,
            },
        },
        // IndexFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02282, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // MiddleFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00231, 0.00559, 0.01632, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.55347,
                x: -0.51293,
                y: 0.49363,
                z: 0.43232,
            },
        },
        // MiddleFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.07089, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.70241,
                x: -0.01896,
                y: 0.03782,
                z: 0.71051,
            },
        },
        // MiddleFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04311, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.75555,
                x: -0.03905,
                y: 0.03900,
                z: 0.65276,
            },
        },
        // MiddleFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.03327, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.74411,
                x: -0.01136,
                y: -0.06222,
                z: 0.66506,
            },
        },
        // MiddleFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02589, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: -0.00000,
                z: 0.00000,
            },
        },
        // RingFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00051, -0.00655, 0.01635, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.55014,
                x: -0.51669,
                y: 0.42989,
                z: 0.49555,
            },
        },
        // RingFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.06537, -0.00070, 0.00205, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.70078,
                x: 0.07535,
                y: 0.01841,
                z: 0.70915,
            },
        },
        // RingFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.04033, -0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.71382,
                x: 0.00833,
                y: 0.07700,
                z: 0.69603,
            },
        },
        // RingFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02849, 0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.78821,
                x: 0.05548,
                y: 0.13836,
                z: 0.59708,
            },
        },
        // RingFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02243, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: 0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // PinkyFinger0
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.00363, -0.01205, 0.01982, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.51533,
                x: -0.48576,
                y: 0.34675,
                z: 0.61502,
            },
        },
        // PinkyFinger1
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.06286, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.75876,
                x: 0.16471,
                y: 0.00588,
                z: 0.63017,
            },
        },
        // PinkyFinger2
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.02987, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.70376,
                x: -0.00134,
                y: -0.01480,
                z: 0.71028,
            },
        },
        // PinkyFinger3
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01798, 0.00000, -0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.82872,
                x: -0.00344,
                y: 0.00593,
                z: 0.55962,
            },
        },
        // PinkyFinger4
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [-0.01802, -0.00000, 0.00000, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 1.00000,
                x: -0.00000,
                y: 0.00000,
                z: -0.00000,
            },
        },
        // Aux_Thumb
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03928, 0.06008, 0.08449, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.56911,
                x: 0.04861,
                y: 0.81959,
                z: 0.04504,
            },
        },
        // Aux_IndexFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03767, 0.02825, 0.06347, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.57495,
                x: -0.76383,
                y: -0.21599,
                z: -0.19834,
            },
        },
        // Aux_MiddleFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03942, 0.00844, 0.05936, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.67319,
                x: -0.71387,
                y: -0.15013,
                z: -0.12112,
            },
        },
        // Aux_RingFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.03506, -0.01146, 0.05747, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.80517,
                x: -0.55694,
                y: -0.13109,
                z: -0.15599,
            },
        },
        // Aux_PinkyFinger
        vr::VRBoneTransform_t {
            position: vr::HmdVector4_t {
                v: [0.02885, -0.03045, 0.06792, 1.0],
            },
            orientation: vr::HmdQuaternionf_t {
                w: 0.69794,
                x: -0.65475,
                y: -0.18023,
                z: -0.22736,
            },
        },
    ];
}
