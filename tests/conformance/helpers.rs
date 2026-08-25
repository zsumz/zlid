use std::cmp::Ordering;

use zlid::{ClockState, Profile, SentinelName};

pub(crate) fn parse_profile(value: &str) -> Profile {
    Profile::from_wire_name(value).unwrap()
}

pub(crate) fn parse_clock_state(value: &str) -> ClockState {
    match value {
        "normal" => ClockState::Normal,
        "clamped" => ClockState::Clamped,
        other => panic!("unknown clock state {other}"),
    }
}

pub(crate) fn parse_sentinel_name(value: &str) -> SentinelName {
    match value {
        "NIL" => SentinelName::Nil,
        "MAX" => SentinelName::Max,
        other => panic!("unknown sentinel name {other}"),
    }
}

pub(crate) fn ordering_sign(ordering: Ordering) -> i8 {
    match ordering {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}
