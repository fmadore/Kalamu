//! What an address the researcher typed says about where it leads: whether Kalamu takes it
//! ([`checked_address`]), its host, and whether that host is this computer. One rule, read by
//! the catalogue (what `servers.json` may hold, what the interface shows) and by the providers
//! (what discovery accepts, what consent asks), so the two can never disagree
//! ([decision](../../../docs/decisions/2026-11-26-a-server-of-the-researchers-own.md)).
//!
//! The host is read by hand, without a URL library: only the host is wanted, and every doubt is
//! resolved towards "on the network", which asks consent, never towards "on this computer",
//! which does not. An address a URL library would read as a loopback address in an unusual
//! spelling (`127.1`, a host written in percent escapes) is therefore taken for the network:
//! consent is asked where it was not needed, never skipped where it was. [`checked_address`]
//! writes such an address as the URL library reads it (`127.1` becomes `127.0.0.1`), so the
//! address kept is the one requests go to.

use std::net::{Ipv4Addr, Ipv6Addr};

use thiserror::Error;
use url::Url;

/// Why an address typed by the researcher is not taken. Each has a stable [`code`](Self::code)
/// the interface words.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AddressError {
    #[error("Type the server's address, such as http://localhost:11434")]
    Empty,
    #[error("This is not an address Kalamu can read; write it as http://localhost:11434")]
    NotAnAddress,
    #[error("The address must start with http:// or https://")]
    Scheme,
    #[error(
        "Leave the user name and password out of the address; if the server asks for a key, \
         tick “This server asks for a key”"
    )]
    Credentials,
    #[error("Leave out what follows ? or # in the address")]
    Query,
}

impl AddressError {
    /// The id the interface looks the sentence up by.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "server.address_empty",
            Self::NotAnAddress => "server.address_unreadable",
            Self::Scheme => "server.address_scheme",
            Self::Credentials => "server.address_credentials",
            Self::Query => "server.address_query",
        }
    }
}

/// An address as Kalamu keeps it: trimmed, `http` or `https` only, with a host, without a user
/// name or password, a query or a fragment, without a space, a control character or a
/// backslash inside it, and without a trailing slash. A path is kept, since a gateway may live
/// under one. The host is written as the URL library reads it (in lower case, an IPv4 address
/// in its usual form), the address requests go to.
///
/// What it returns, it takes again unchanged; and an address `servers.json` holds is refused
/// by the catalogue exactly when this refuses it.
pub fn checked_address(input: &str) -> Result<String, AddressError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(AddressError::Empty);
    }
    // The URL library would drop a tab or a line break, and turn a backslash into a slash:
    // what is kept must be what was typed.
    if input
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return Err(AddressError::NotAnAddress);
    }
    let url = Url::parse(input).map_err(|_| AddressError::NotAnAddress)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AddressError::Scheme);
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err(AddressError::NotAnAddress);
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AddressError::Credentials);
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(AddressError::Query);
    }
    Ok(url.as_str().trim_end_matches('/').to_string())
}

/// The host of an address, in lower case, without the brackets of an IPv6 address, its port, a
/// user name before it or a path after it: `http://User@LocalHost:11434/v1` gives `localhost`,
/// `http://[::1]:8080` gives `::1`. An address without a scheme is read from its start. Empty
/// when the address names no host.
pub fn host_of(address: &str) -> String {
    let address = address.trim();
    let rest = match address.find("://") {
        Some(at) => &address[at + 3..],
        None => address,
    };
    // A backslash ends the authority too, as browsers and URL libraries read `http` addresses.
    let authority = rest.split(['/', '\\', '?', '#']).next().unwrap_or_default();
    let host_port = authority.rsplit('@').next().unwrap_or_default();
    let host = if let Some(bracketed) = host_port.strip_prefix('[') {
        bracketed.split(']').next().unwrap_or_default()
    } else {
        host_port.split(':').next().unwrap_or_default()
    };
    host.to_ascii_lowercase()
}

/// Whether an address leads to this computer: the host `localhost` (with or without a final
/// dot), any address of `127.0.0.0/8`, or `::1` (also written as an IPv4 address in IPv6,
/// `::ffff:127.0.0.1`). Anything else, a name that happens to lead back here included, is taken
/// for the network. An address holding a space or a control character is never this computer.
pub fn on_this_computer(address: &str) -> bool {
    if address
        .trim()
        .chars()
        .any(|c| c.is_whitespace() || c.is_control())
    {
        return false;
    }
    let host = host_of(address);
    let name = host.strip_suffix('.').unwrap_or(&host);
    if name == "localhost" {
        return true;
    }
    if let Ok(v4) = host.parse::<Ipv4Addr>() {
        return v4.is_loopback();
    }
    if let Ok(v6) = host.parse::<Ipv6Addr>() {
        return v6.is_loopback() || v6.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback());
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_is_kept_trimmed_without_its_last_slash_and_as_requests_read_it() {
        for (typed, kept) in [
            ("  http://localhost:11434/ ", "http://localhost:11434"),
            (
                "https://gpu.example.edu/llm/v1/",
                "https://gpu.example.edu/llm/v1",
            ),
            ("HTTP://LocalHost:11434", "http://localhost:11434"),
            ("http://[::1]:11434", "http://[::1]:11434"),
            ("http://127.1:11434", "http://127.0.0.1:11434"),
            ("https://host:443/v1", "https://host/v1"),
        ] {
            assert_eq!(checked_address(typed).as_deref(), Ok(kept), "{typed}");
        }
    }

    #[test]
    fn an_address_kalamu_cannot_keep_is_refused_for_its_reason() {
        for (typed, why) in [
            ("", AddressError::Empty),
            ("   ", AddressError::Empty),
            ("ftp://host", AddressError::Scheme),
            ("file:///etc/passwd", AddressError::Scheme),
            ("localhost:11434", AddressError::Scheme),
            ("http://me:secret@host:11434", AddressError::Credentials),
            ("http://me@host", AddressError::Credentials),
            ("http://host/?key=1", AddressError::Query),
            ("http://host/#x", AddressError::Query),
            ("not an address", AddressError::NotAnAddress),
            ("http://", AddressError::NotAnAddress),
            ("http://:11434", AddressError::NotAnAddress),
            ("http://host:99999", AddressError::NotAnAddress),
            ("http://server lan", AddressError::NotAnAddress),
            ("http://local\nhost", AddressError::NotAnAddress),
            ("http://server.lan\\@localhost/", AddressError::NotAnAddress),
        ] {
            assert_eq!(checked_address(typed), Err(why), "{typed:?}");
        }
    }

    #[test]
    fn what_is_kept_is_taken_again_unchanged_and_never_changes_where_it_leads() {
        for typed in [
            "http://localhost:11434/",
            "HTTP://LOCALHOST:11434",
            "http://127.1",
            "http://[0:0:0:0:0:0:0:1]:11434",
            "https://gpu.example.edu/llm/v1/",
            "http://192.168.1.20:11434",
        ] {
            let kept = checked_address(typed).unwrap_or_default();
            assert_eq!(
                checked_address(&kept).as_deref(),
                Ok(kept.as_str()),
                "{typed}"
            );
            assert!(!kept.is_empty() && !host_of(&kept).is_empty(), "{typed}");
        }
        // Read by hand, `127.1` is the network; kept, it is the loopback address it leads to.
        assert!(!on_this_computer("http://127.1"));
        assert!(on_this_computer(
            &checked_address("http://127.1").unwrap_or_default()
        ));
    }

    #[test]
    fn the_host_is_read_without_scheme_user_port_or_path() {
        assert_eq!(host_of("http://localhost:11434"), "localhost");
        assert_eq!(
            host_of("https://LLM.Uni-Example.org/v1/"),
            "llm.uni-example.org"
        );
        assert_eq!(host_of("http://user:secret@host.lan:80/x"), "host.lan");
        assert_eq!(host_of("http://[::1]:11434/api"), "::1");
        assert_eq!(host_of("http://[::1]"), "::1");
        assert_eq!(host_of("192.168.1.20:11434"), "192.168.1.20");
        assert_eq!(host_of("http://host?x=1"), "host");
        assert_eq!(host_of("http://host#part"), "host");
        assert_eq!(host_of("  http://host.lan  "), "host.lan");
        assert_eq!(host_of("http://"), "");
        assert_eq!(host_of(""), "");
    }

    #[test]
    fn this_computer_is_localhost_the_loopback_range_and_ipv6_loopback() {
        for local in [
            "http://localhost:11434",
            "http://LOCALHOST:11434",
            "http://localhost.:11434",
            "https://localhost/v1",
            "http://127.0.0.1:11434",
            "http://127.0.0.1",
            "http://127.255.10.2:8080",
            "http://[::1]:11434",
            "http://[0:0:0:0:0:0:0:1]:11434",
            "http://[::ffff:127.0.0.1]:11434",
            "localhost:11434",
            "http://user@localhost:11434",
        ] {
            assert!(on_this_computer(local), "{local}");
        }
    }

    #[test]
    fn every_other_address_is_the_network() {
        for network in [
            "http://192.168.1.20:11434",
            "http://10.0.0.2",
            "http://0.0.0.0:11434",
            "http://128.0.0.1",
            "https://llm.uni-example.org",
            "http://localhost.example.org",
            "http://mylocalhost",
            "http://localhost..",
            "http://127.0.0.1.nip.io",
            // Short and decimal spellings of 127.0.0.1 are left to the network: consent is
            // asked where it was not needed, never skipped.
            "http://127.1",
            "http://2130706433",
            "http://[::2]",
            "http://[fe80::1]",
            "http://[::ffff:10.0.0.1]",
            // The host after the user name is the one reached.
            "http://localhost@server.lan",
            "http://localhost:x@server.lan:11434",
            // A backslash ends the host, as URL libraries read it.
            "http://server.lan\\@localhost/",
            // Spaces and control characters are never this computer.
            "http://local host",
            "http://localhost\n.evil.org",
            "",
            "http://",
        ] {
            assert!(!on_this_computer(network), "{network:?}");
        }
    }
}
