use std::convert::TryFrom;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use bytes::Bytes;

use super::{ErrorKind, InvalidUri};
use crate::byte_str::ByteStr;

/// Represents the scheme component of a URI
#[derive(Clone)]
pub struct Scheme {
    pub(super) inner: Scheme2,
}

#[derive(Clone, Debug)]
pub(super) enum Scheme2<T = Box<ByteStr>> {
    None,
    Standard(Protocol),
    Other(T),
}

#[derive(Copy, Clone, Debug)]
pub(super) enum Protocol {
    #[cfg(feature = "http")]
    Http,
    #[cfg(feature = "http")]
    Https,
    #[cfg(feature = "rtsp")]
    Rtsp,
    #[cfg(feature = "rtsp")]
    Rtsps,
}

impl Scheme {
    /// HTTP protocol scheme
    #[cfg(feature = "http")]
    pub const HTTP: Scheme = Scheme {
        inner: Scheme2::Standard(Protocol::Http),
    };

    /// HTTP protocol over TLS.
    #[cfg(feature = "http")]
    pub const HTTPS: Scheme = Scheme {
        inner: Scheme2::Standard(Protocol::Https),
    };

    /// RTSP protocol scheme
    #[cfg(feature = "rtsp")]
    pub const RTSP: Scheme = Scheme {
        inner: Scheme2::Standard(Protocol::Rtsp),
    };

    /// RTSPS protocol scheme
    #[cfg(feature = "rtsp")]
    pub const RTSPS: Scheme = Scheme {
        inner: Scheme2::Standard(Protocol::Rtsps),
    };

    pub(super) fn empty() -> Self {
        Scheme {
            inner: Scheme2::None,
        }
    }

    /// Return a str representation of the scheme
    ///
    /// # Examples
    ///
    /// ```
    /// # use httplike::uri::*;
    /// let scheme: Scheme = "http".parse().unwrap();
    /// assert_eq!(scheme.as_str(), "http");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        use self::Protocol::*;
        use self::Scheme2::*;

        match self.inner {
            #[cfg(feature = "http")]
            Standard(Http) => "http",
            #[cfg(feature = "http")]
            Standard(Https) => "https",
            #[cfg(feature = "rtsp")]
            Standard(Rtsp) => "rtsp",
            #[cfg(feature = "rtsp")]
            Standard(Rtsps) => "rtsps",
            Other(ref v) => &v[..],
            None => unreachable!(),
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for Scheme {
    type Error = InvalidUri;
    #[inline]
    fn try_from(s: &'a [u8]) -> Result<Self, Self::Error> {
        use self::Scheme2::*;

        match Scheme2::parse_exact(s)? {
            None => Err(ErrorKind::InvalidScheme.into()),
            Standard(p) => Ok(Standard(p).into()),
            Other(_) => {
                // Unsafe: parse_exact already checks for a strict subset of UTF-8
                Ok(Other(Box::new(unsafe {
                    ByteStr::from_utf8_unchecked(Bytes::copy_from_slice(s))
                })).into())
            }
        }
    }
}

impl<'a> TryFrom<&'a str> for Scheme {
    type Error = InvalidUri;
    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        TryFrom::try_from(s.as_bytes())
    }
}

impl FromStr for Scheme {
    type Err = InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TryFrom::try_from(s)
    }
}

impl fmt::Debug for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for Scheme {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for Scheme {
    fn eq(&self, other: &Scheme) -> bool {
        use self::Protocol::*;
        use self::Scheme2::*;

        match (&self.inner, &other.inner) {
            #[cfg(feature = "http")]
            (&Standard(Http), &Standard(Http)) => true,
            #[cfg(feature = "http")]
            (&Standard(Https), &Standard(Https)) => true,
            #[cfg(feature = "rtsp")]
            (&Standard(Rtsp), &Standard(Rtsp)) => true,
            #[cfg(feature = "rtsp")]
            (&Standard(Rtsps), &Standard(Rtsps)) => true,
            (&Other(ref a), &Other(ref b)) => a.eq_ignore_ascii_case(b),
            (&None, _) | (_, &None) => unreachable!(),
            _ => false,
        }
    }
}

impl Eq for Scheme {}

/// Case-insensitive equality
///
/// # Examples
///
/// ```
/// # use httplike::uri::Scheme;
/// let scheme: Scheme = "HTTP".parse().unwrap();
/// assert_eq!(scheme, *"http");
/// ```
impl PartialEq<str> for Scheme {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq_ignore_ascii_case(other)
    }
}

/// Case-insensitive equality
impl PartialEq<Scheme> for str {
    fn eq(&self, other: &Scheme) -> bool {
        other == self
    }
}

/// Case-insensitive hashing
impl Hash for Scheme {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self.inner {
            Scheme2::None => (),
            #[cfg(feature = "http")]
            Scheme2::Standard(Protocol::Http) => state.write_u8(1),
            #[cfg(feature = "http")]
            Scheme2::Standard(Protocol::Https) => state.write_u8(2),
            #[cfg(feature = "rtsp")]
            Scheme2::Standard(Protocol::Rtsp) => state.write_u8(3),
            #[cfg(feature = "rtsp")]
            Scheme2::Standard(Protocol::Rtsps) => state.write_u8(4),
            Scheme2::Other(ref other) => {
                other.len().hash(state);
                for &b in other.as_bytes() {
                    state.write_u8(b.to_ascii_lowercase());
                }
            }
        }
    }
}

impl<T> Scheme2<T> {
    pub(super) fn is_none(&self) -> bool {
        match *self {
            Scheme2::None => true,
            _ => false,
        }
    }
}

// Require the scheme to not be too long in order to enable further
// optimizations later.
const MAX_SCHEME_LEN: usize = 64;

// scheme = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
//
const SCHEME_CHARS: [u8; 256] = [
    //  0      1      2      3      4      5      6      7      8      9
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //   x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  1x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  2x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  3x
        0,     0,     0,  b'+',     0,  b'-',  b'.',     0,  b'0',  b'1', //  4x
     b'2',  b'3',  b'4',  b'5',  b'6',  b'7',  b'8',  b'9',  b':',     0, //  5x
        0,     0,     0,     0,     0,  b'A',  b'B',  b'C',  b'D',  b'E', //  6x
     b'F',  b'G',  b'H',  b'I',  b'J',  b'K',  b'L',  b'M',  b'N',  b'O', //  7x
     b'P',  b'Q',  b'R',  b'S',  b'T',  b'U',  b'V',  b'W',  b'X',  b'Y', //  8x
     b'Z',     0,     0,     0,     0,     0,     0,  b'a',  b'b',  b'c', //  9x
     b'd',  b'e',  b'f',  b'g',  b'h',  b'i',  b'j',  b'k',  b'l',  b'm', // 10x
     b'n',  b'o',  b'p',  b'q',  b'r',  b's',  b't',  b'u',  b'v',  b'w', // 11x
     b'x',  b'y',  b'z',     0,     0,     0,  b'~',     0,     0,     0, // 12x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 13x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 14x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 15x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 16x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 17x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 18x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 19x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 20x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 21x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 22x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 23x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 24x
        0,     0,     0,     0,     0,     0                              // 25x
];

impl Scheme2<usize> {
    fn parse_exact(s: &[u8]) -> Result<Scheme2<()>, InvalidUri> {
        match s {
            #[cfg(feature = "http")]
            b"http" => Ok(Protocol::Http.into()),
            #[cfg(feature = "http")]
            b"https" => Ok(Protocol::Https.into()),
            #[cfg(feature = "rtsp")]
            b"rtsp" => Ok(Protocol::Rtsp.into()),
            #[cfg(feature = "rtsp")]
            b"rtsps" => Ok(Protocol::Rtsps.into()),
            _ => {
                if s.len() > MAX_SCHEME_LEN {
                    return Err(ErrorKind::SchemeTooLong.into());
                }

                for &b in s {
                    match SCHEME_CHARS[b as usize] {
                        b':' => {
                            // Don't want :// here
                            return Err(ErrorKind::InvalidScheme.into());
                        }
                        0 => {
                            return Err(ErrorKind::InvalidScheme.into());
                        }
                        _ => {}
                    }
                }

                Ok(Scheme2::Other(()))
            }
        }
    }

    pub(super) fn parse(s: &[u8]) -> Result<Scheme2<usize>, InvalidUri> {
        if s.len() >= 7 {
            #[cfg(feature = "http")]
            {
                // Check for HTTP
                if s[..7].eq_ignore_ascii_case(b"http://") {
                    // Prefix will be striped
                    return Ok(Protocol::Http.into());
                }
            }
            #[cfg(feature = "rtsp")]
            {
                // Check for RTSP
                if s[..7].eq_ignore_ascii_case(b"rtsp://") {
                    // Prefix will be striped
                    return Ok(Protocol::Rtsp.into());
                }
            }
        }

        if s.len() >= 8 {
            #[cfg(feature = "http")]
            {
                // Check for HTTPs
                if s[..8].eq_ignore_ascii_case(b"https://") {
                    return Ok(Protocol::Https.into());
                }
            }
            #[cfg(feature = "rtsp")]
            {
                // Check for RTSPs
                if s[..8].eq_ignore_ascii_case(b"rtsps://") {
                    return Ok(Protocol::Rtsps.into());
                }
            }
        }

        if s.len() > 3 {
            for i in 0..s.len() {
                let b = s[i];

                match SCHEME_CHARS[b as usize] {
                    b':' => {
                        // Not enough data remaining
                        if s.len() < i + 3 {
                            break;
                        }

                        // Not a scheme
                        if &s[i + 1..i + 3] != b"//" {
                            break;
                        }

                        if i > MAX_SCHEME_LEN {
                            return Err(ErrorKind::SchemeTooLong.into());
                        }

                        // Return scheme
                        return Ok(Scheme2::Other(i));
                    }
                    // Invald scheme character, abort
                    0 => break,
                    _ => {}
                }
            }
        }

        Ok(Scheme2::None)
    }
}

impl Protocol {
    pub(super) fn len(&self) -> usize {
        match *self {
            #[cfg(feature = "http")]
            Protocol::Http => 4,
            #[cfg(feature = "http")]
            Protocol::Https => 5,
            #[cfg(feature = "rtsp")]
            Protocol::Rtsp => 4,
            #[cfg(feature = "rtsp")]
            Protocol::Rtsps => 5,
        }
    }
}

impl<T> From<Protocol> for Scheme2<T> {
    fn from(src: Protocol) -> Self {
        Scheme2::Standard(src)
    }
}

#[doc(hidden)]
impl From<Scheme2> for Scheme {
    fn from(src: Scheme2) -> Self {
        Scheme { inner: src }
    }
}
