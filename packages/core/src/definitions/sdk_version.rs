use crate::{encoding::SimpleParseResult, prelude::*};

pub fn parse_libary_version(version: &str) -> SimpleParseResult<Version> {
    let version = if version.starts_with("Z-Wave ") {
        version.split_at("Z-Wave ".len()).1
    } else {
        version
    };
    Version::try_from(version)
}

pub fn protocol_version_to_sdk_version(protocol_version: &Version) -> Version {
    match (protocol_version.major, protocol_version.minor) {
        (6, 10) => Version::try_from("6.84.0").unwrap(), // Serial API Version: 8
        (6, 9) => Version::try_from("6.82.1").unwrap(),  // Serial API Version: 8
        (6, 8) => Version::try_from("6.82.0").unwrap(),  // Serial API Version: 8
        (6, 7) => Version::try_from("6.81.6").unwrap(),  // Serial API Version: 8
        (6, 6) => Version::try_from("6.81.5").unwrap(),  // Serial API Version: 8
        (6, 5) => Version::try_from("6.81.4").unwrap(),  // Serial API Version: 8
        (6, 4) => Version::try_from("6.81.3").unwrap(),  // Serial API Version: 8
        (6, 3) => Version::try_from("6.81.2").unwrap(),  // Serial API Version: 8
        (6, 2) => Version::try_from("6.81.1").unwrap(),  // Serial API Version: 8
        (6, 1) => Version::try_from("6.81.0").unwrap(),  // Serial API Version: 8
        // (6, 1) => Version::try_from("6.80.0-beta").unwrap(), // Serial API Version: 8
        (5, 3) => Version::try_from("6.71.3").unwrap(),  // Serial API Version: 7
        (5, 2) => Version::try_from("6.71.2").unwrap(),  // Serial API Version: 7
        (4, 61) => Version::try_from("6.71.1").unwrap(), // Serial API Version: 7
        (4, 60) => Version::try_from("6.71.0").unwrap(), // Serial API Version: 7
        (4, 45) => Version::try_from("6.70.1").unwrap(), // Serial API Version: 6
        (4, 28) => Version::try_from("6.70.0").unwrap(), // Serial API Version: 6
        (4, 62) => Version::try_from("6.61.1").unwrap(), // Serial API Version: 6
        (4, 33) => Version::try_from("6.61.0").unwrap(), // Serial API Version: 6
        (4, 12) => Version::try_from("6.60.0").unwrap(), // Serial API Version: 6
        (4, 54) => Version::try_from("6.51.10").unwrap(), // Serial API Version: 5
        (4, 38) => Version::try_from("6.51.9").unwrap(), // Serial API Version: 5
        (4, 34) => Version::try_from("6.51.8").unwrap(), // Serial API Version: 5
        (4, 24) => Version::try_from("6.51.7").unwrap(), // Serial API Version: 5
        (4, 5) => Version::try_from("6.51.6").unwrap(),  // Serial API Version: 5
        (4, 1) => Version::try_from("6.51.4").unwrap(),  // Serial API Version: 5
        (3, 99) => Version::try_from("6.51.3").unwrap(), // Serial API Version: 5
        (3, 95) => Version::try_from("6.51.2").unwrap(), // Serial API Version: 5
        (3, 92) => Version::try_from("6.51.1").unwrap(), // Serial API Version: 5
        (3, 83) => Version::try_from("6.51.0").unwrap(), // Serial API Version: 5
        (3, 79) => Version::try_from("6.50.1").unwrap(), // Serial API Version: 5
        (3, 71) => Version::try_from("6.50.0").unwrap(), // Serial API Version: 5
        // The entries below this line are for the 300 or 400 series
        (3, 53) => Version::try_from("6.11.1").unwrap(), // Serial API Version: 5 / JP only
        (3, 45) => Version::try_from("6.11.0").unwrap(), // Serial API Version: 5 / JP only
        (3, 38) => Version::try_from("6.10.1").unwrap(), // Serial API Version: 5 / JP only
        (3, 35) => Version::try_from("6.10.0").unwrap(), // Serial API Version: 5
        (3, 41) => Version::try_from("6.2.0").unwrap(),  // Serial API Version: 5
        (3, 37) => Version::try_from("6.1.3").unwrap(),  // Serial API Version: 5
        (3, 33) => Version::try_from("6.1.2").unwrap(),  // Serial API Version: 5
        (3, 26) => Version::try_from("6.1.1").unwrap(),  // Serial API Version: 5 / 2-ch
        (3, 10) => Version::try_from("6.1.0").unwrap(),  // Serial API Version: 5
        (3, 7) => Version::try_from("6.0.5").unwrap(), // Serial API Version: 5
        (3, 6) => Version::try_from("6.0.4").unwrap(), // Serial API Version: 5
        (3, 4) => Version::try_from("6.0.3").unwrap(), // Serial API Version: 5
        (3, 3) => Version::try_from("6.0.2").unwrap(), // Serial API Version: 5
        (2, 99) => Version::try_from("6.0.1").unwrap(), // Serial API Version: 5
        (2, 96) => Version::try_from("6.0.0").unwrap(), // Serial API Version: 5
        (3, 28) => Version::try_from("5.3.0").unwrap(),  // Serial API Version: 5
        (2, 78) => Version::try_from("5.2.3").unwrap(),  // Serial API Version: 5
        (2, 64) => Version::try_from("5.2.2").unwrap(),  // Serial API Version: 5
        (2, 51) => Version::try_from("5.2.1").unwrap(),  // Serial API Version: 5
        (2, 48) => Version::try_from("5.2.0").unwrap(),  // Serial API Version: 5
        (2, 36) => Version::try_from("5.1.0").unwrap(),  // Serial API Version: 5
        (2, 22) => Version::try_from("5.0.1").unwrap(), // Serial API Version: 5
        (2, 16) => Version::try_from("5.0.0").unwrap(), // Serial API Version: 5
        (3, 67) => Version::try_from("4.55.0").unwrap(), // Serial API Version: 5
        (3, 52) => Version::try_from("4.54.2").unwrap(), // Serial API Version: 5
        (3, 42) => Version::try_from("4.54.1").unwrap(), // Serial API Version: 5
        (3, 40) => Version::try_from("4.54.0").unwrap(), // Serial API Version: 5
        (3, 36) => Version::try_from("4.53.1").unwrap(), // Serial API Version: 5
        (3, 34) => Version::try_from("4.53.0").unwrap(), // Serial API Version: 5
        (3, 22) => Version::try_from("4.52.1").unwrap(), // Serial API Version: 5
        (3, 20) => Version::try_from("4.52.0").unwrap(), // Serial API Version: 5
        (2, 97) => Version::try_from("4.51.0").unwrap(), // Serial API Version: 5
        // Unknown OR 700+ series
        _ => *protocol_version,
    }
}
