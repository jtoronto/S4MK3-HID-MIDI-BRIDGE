// Root's prearmed NIHWMainHandler probe received this exact 63-byte message.
pub(super) const CONNECT_REPLY: &[u8] = b"eurt\x15\0\0\0NIHWS17200045Request\0\
    \x1a\0\0\0NIHWS17200045Notification\0\0\0\0\0";

// Root's production worker captured this complete reply at 22:32:33 UTC.
pub(super) const CONNECT_REPLY_OPAQUE: &[u8] = b"eurt\x15\0\0\0NIHWS17200049Request\0\
    \x1a\0\0\0NIHWS17200049Notification\0WTra";
