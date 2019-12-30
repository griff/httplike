//! HTTP version
//!
//! This module contains a definition of the `Version` type. The `Version`
//! type is intended to be accessed through the root of the crate
//! (`httplike::Version`) rather than this module.
//!
//! The `Version` type contains constants that represent the various versions
//! of the HTTP protocol.
//!
//! # Examples
//!
//! ```
//! use httplike::Version;
//!
//! let http11 = Version::HTTP_11;
//! let http2 = Version::HTTP_2;
//! assert!(http11 != http2);
//!
//! println!("{:?}", http2);
//! ```

use std::fmt;

/// Represents a version of the HTTP spec.
#[derive(PartialEq, PartialOrd, Copy, Clone, Eq, Ord, Hash)]
pub struct Version(Protocol);

impl Version {
    /// `HTTP/0.9`
    #[cfg(feature = "http")]
    pub const HTTP_09: Version = Version(Protocol::Http09);

    /// `HTTP/1.0`
    #[cfg(feature = "http")]
    pub const HTTP_10: Version = Version(Protocol::Http10);

    /// `HTTP/1.1`
    #[cfg(feature = "http")]
    pub const HTTP_11: Version = Version(Protocol::Http11);

    /// `HTTP/2.0`
    #[cfg(feature = "http")]
    pub const HTTP_2: Version = Version(Protocol::H2);

    /// `HTTP/3.0`
    #[cfg(feature = "http")]
    pub const HTTP_3: Version = Version(Protocol::H3);

    /// `RTSP/1.0`
    #[cfg(feature = "rtsp")]
    pub const RTSP_1: Version = Version(Protocol::Rtsp1);
}

#[derive(PartialEq, PartialOrd, Copy, Clone, Eq, Ord, Hash)]
enum Protocol {
    #[cfg(feature = "http")]
    Http09,
    #[cfg(feature = "http")]
    Http10,
    #[cfg(feature = "http")]
    Http11,
    #[cfg(feature = "http")]
    H2,
    #[cfg(feature = "http")]
    H3,
    #[cfg(feature = "rtsp")]
    Rtsp1,
    __NonExhaustive,
}

impl Default for Version {
    #[inline]
    #[cfg(feature = "http")]
    fn default() -> Version {
        Version::HTTP_11
    }

    #[inline]
    #[cfg(all(not(feature = "http"), feature = "rtsp"))]
    fn default() -> Version {
        Version::RTSP_1
    }

    #[inline]
    #[cfg(all(not(feature = "http"), not(feature = "rtsp"), feature="sip"))]
    fn default() -> Version {
        Version::RTSP_1
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Protocol::*;

        f.write_str(match self.0 {
            #[cfg(feature = "http")]
            Http09 => "HTTP/0.9",
            #[cfg(feature = "http")]
            Http10 => "HTTP/1.0",
            #[cfg(feature = "http")]
            Http11 => "HTTP/1.1",
            #[cfg(feature = "http")]
            H2 => "HTTP/2.0",
            #[cfg(feature = "http")]
            H3 => "HTTP/3.0",
            #[cfg(feature = "rtsp")]
            Rtsp1  => "RTSP/1.0",
            __NonExhaustive => unreachable!(),
        })
    }
}
