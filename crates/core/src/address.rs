//! What an address the researcher typed says about where it leads: its host, and whether that
//! host is this computer. One rule, read by the catalogue (what the interface shows) and by the
//! providers (what consent asks), so the two can never disagree
//! ([decision](../../../docs/decisions/2026-11-26-a-server-of-the-researchers-own.md)).
//!
//! Read by hand, without a URL library: only the host is wanted, and every doubt is resolved
//! towards "on the network", which asks consent, never towards "on this computer", which does
//! not. An address a URL library would read as a loopback address in an unusual spelling
//! (`127.1`, a host written in percent escapes) is therefore taken for the network: consent is
//! asked where it was not needed, never skipped where it was.

use std::net::{Ipv4Addr, Ipv6Addr};

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
