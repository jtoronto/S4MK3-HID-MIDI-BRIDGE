pub(super) fn read(
    operation: u64,
    report: &mut [u8],
    read: impl FnOnce(&mut [u8]) -> hidapi::HidResult<usize>,
) -> hidapi::HidResult<usize> {
    use s4mk3_hid_midi_bridge::screen_motor_diag::{self as diag, Kind};
    let begin = diag::record(
        Kind::InputBegin,
        [
            operation,
            u64::try_from(report.len()).unwrap_or(u64::MAX),
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        &[],
    );
    let result = read(report);
    let length = result.as_ref().copied().unwrap_or(0);
    diag::record(
        Kind::InputEnd,
        [
            begin,
            operation,
            u64::from(result.is_ok()),
            u64::try_from(length).unwrap_or(u64::MAX),
            u64::from(report.first().copied().unwrap_or(0)),
            0,
            0,
            0,
        ],
        &report[..length.min(report.len()).min(95)],
    );
    result
}
