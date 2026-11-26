use serde::{Deserialize, Serialize};

/// The diagnostic screen traffic case.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Case {
    #[default]
    Normal,
    Off,
    Acquired,
    Static,
    Updates,
}

/// The single producer assigned to a diagnostic role.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Output,
    Input,
    Screen,
}

impl Role {
    pub(super) const fn index(self) -> usize {
        match self {
            Self::Output => 0,
            Self::Input => 1,
            Self::Screen => 2,
        }
    }
}

/// Event identity; the numeric values are interpreted by the caller.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    InputBegin,
    InputEnd,
    BatchPublished,
    BatchConsumed,
    Sensor,
    MotorState,
    Decision,
    Playback,
    Feedback,
    InputTimeout,
    InputRecovered,
    MotionReset,
    NativeWriteBegin,
    NativeWriteEnd,
    ScreenConnectBegin,
    ScreenReady,
    ScreenRenderBegin,
    ScreenRenderEnd,
    ScreenPublishBegin,
    ScreenPublishEnd,
    ScreenReleaseBegin,
    ScreenReleaseEnd,
    ScreenCaseReady,
    Lifecycle,
}

#[derive(Clone, Copy)]
pub(super) struct Event {
    pub us: u64,
    pub seq: u64,
    pub kind: Kind,
    pub values: [u64; 8],
    pub payload: [u8; 95],
    pub payload_len: u8,
}
