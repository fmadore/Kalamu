//! Addresses as the researcher types them: checked and placed (on this computer or on the
//! network) by `kalamu_core::address`, the one rule the catalogue reads too; here only how a
//! message names the host, with its port.

use reqwest::Url;

pub use kalamu_core::address::{AddressError, checked_address, on_this_computer};

/// The host of an address, with its port, as messages name it: `192.168.1.20:11434`,
/// `[::1]:11434`, `gpu.example.edu`. The host is `kalamu_core::address::host_of`'s; an address
/// that names none is named whole.
pub fn host_of(address: &str) -> String {
    let host = kalamu_core::address::host_of(address);
    if host.is_empty() {
        return address.trim().to_string();
    }
    let host = if host.contains(':') {
        format!("[{host}]")
    } else {
        host
    };
    match Url::parse(address.trim()).ok().and_then(|url| url.port()) {
        Some(port) => format!("{host}:{port}"),
        None => host,
    }
}

/// Whether an address's host is `domain` or one of its subdomains.
pub(super) fn host_is(address: &str, domain: &str) -> bool {
    let host = kalamu_core::address::host_of(address);
    host == domain
        || host
            .strip_suffix(domain)
            .is_some_and(|sub| sub.ends_with('.'))
}
