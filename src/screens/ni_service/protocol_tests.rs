use super::*;
use crate::screens::ni_service::fixtures::{CONNECT_REPLY, CONNECT_REPLY_OPAQUE};

#[test]
fn solid_display_frames_match_physically_verified_encdr_messages() {
    // Retained Encdr a541465 offline frames from the attended two-screen test.
    for (index, rgb, pair) in [(0, 0x001f, [0xf8, 0]), (1, 0xf800, [0, 0x1f])] {
        let mut actual = Vec::new();
        display(index, &vec![rgb; PIXELS], PixelOrder::Bgr565, &mut actual);
        let expected = [
            0x44, 0x73, 0x64, 3, index, 0, 0, 0, 0, 0, 0, 0, 0xf0, 0, 0x40, 1, 0x20, 0, 0, 0, 0x84,
            0, index, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x40, 0, 0xf0, 1, 0, 0x96, 0, pair[0],
            pair[1], pair[0], pair[1], 3, 0, 0, 0, 0x40, 0, 0, 0,
        ];
        assert_eq!(actual, expected, "display {index}");
    }
}

#[test]
fn actual_bootstrap_replies_parse_port_names_and_optional_opaque_footer() -> Result<(), Error> {
    assert_eq!(CONNECT_REPLY.len(), 63);
    for message in [CONNECT_REPLY, &CONNECT_REPLY[..59]] {
        let ports = ports(message)?;
        assert_eq!(ports.request, "NIHWS17200045Request");
        assert_eq!(ports.notification, "NIHWS17200045Notification");
    }
    let opaque = ports(CONNECT_REPLY_OPAQUE)?;
    assert_eq!(opaque.request, "NIHWS17200049Request");
    assert_eq!(opaque.notification, "NIHWS17200049Notification");
    Ok(())
}

#[test]
fn connect_reply_rejects_truncated_and_unbounded_extensions() {
    for end in 0..CONNECT_REPLY.len() {
        if end != 59 {
            assert!(ports(&CONNECT_REPLY[..end]).is_err(), "truncation {end}");
        }
    }
    for footer in [&[1][..], &[0, 1], &[0, 0, 1], &[0; 5], &[0; 4096]] {
        let mut message = CONNECT_REPLY[..59].to_vec();
        message.extend_from_slice(footer);
        assert!(ports(&message).is_err());
    }
}
