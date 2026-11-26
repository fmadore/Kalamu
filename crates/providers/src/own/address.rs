//! Addresses as the researcher types them, checked in one place, and where they are: on this
//! computer or on the network, decided by the address alone.

use std::net::IpAddr;

use reqwest::Url;
use thiserror::Error;

/// Why an address typed by the researcher is not taken.
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

/// An address as Kalamu keeps it: trimmed, `http` or `https` only, without a user name or
/// password, a query or a fragment, and without a trailing slash. A path is kept, since a
/// gateway may live under one.
pub fn checked_address(input: &str) -> Result<String, AddressError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(AddressError::Empty);
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

/// Whether an address is on this computer: `localhost`, `127.0.0.0/8` or `::1`. Decided by the
/// address alone; anything else, or an address that cannot be read, is on the network.
pub fn on_this_computer(address: &str) -> bool {
    let Ok(url) = Url::parse(address.trim()) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    match bare.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => ip.is_loopback(),
        Ok(IpAddr::V6(ip)) => {
            ip.is_loopback() || ip.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback())
        }
        Err(_) => host.trim_end_matches('.').eq_ignore_ascii_case("localhost"),
    }
}

/// The host of an address, with its port, as messages and consent name it:
/// `192.168.1.20:11434`, `[::1]:11434`, `gpu.example.edu`.
pub fn host_of(address: &str) -> String {
    match Url::parse(address.trim()) {
        Ok(url) => match (url.host_str(), url.port()) {
            (Some(host), Some(port)) => format!("{host}:{port}"),
            (Some(host), None) => host.to_string(),
            _ => address.trim().to_string(),
        },
        Err(_) => address.trim().to_string(),
    }
}

/// Whether an address's host is `domain` or one of its subdomains.
pub(super) fn host_is(address: &str, domain: &str) -> bool {
    Url::parse(address).ok().is_some_and(|url| {
        url.host_str().is_some_and(|host| {
            host == domain
                || host
                    .strip_suffix(domain)
                    .is_some_and(|sub| sub.ends_with('.'))
        })
    })
}
