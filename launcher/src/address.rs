//! The address a phone follows from a code on a television. The server sends it to the device, so
//! it must be one another device can reach.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// This machine's address on the network it would use to leave. Connecting a UDP socket sends
/// nothing, it only makes the system pick a route. The address asked about is a reserved one.
#[must_use]
pub fn lan_ip() -> Option<IpAddr> {
    let any = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
    let socket = UdpSocket::bind(any).ok()?;
    socket
        .connect(SocketAddr::from((Ipv4Addr::new(198, 51, 100, 1), 9)))
        .ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_loopback() && !ip.is_unspecified()).then_some(ip)
}

/// A value for `--site-address`.
///
/// # Errors
/// It must be a whole `http` or `https` address.
pub fn checked(address: &str) -> Result<String> {
    let rest = address
        .strip_prefix("https://")
        .or_else(|| address.strip_prefix("http://"));
    match rest {
        Some(host) if !host.is_empty() => Ok(address.trim_end_matches('/').to_owned()),
        _ => bail!("--site-address needs an address starting with http:// or https://"),
    }
}

/// What was asked, in the order it wins.
#[derive(Clone, Copy, Debug)]
pub struct Site<'a> {
    pub chosen: Option<&'a str>,
    pub domain: &'a str,
    pub public: bool,
    pub port: u16,
    pub lan: Option<IpAddr>,
}

impl Site<'_> {
    /// The address itself. A named address wins, then the brand's domain. Otherwise `--public`
    /// means other devices come in over this network, and without it only this machine can.
    #[must_use]
    pub fn address(&self) -> String {
        if let Some(chosen) = self.chosen {
            return chosen.to_owned();
        }
        if !self.domain.is_empty() {
            return format!("https://{}", self.domain);
        }
        match self.lan.filter(|_| self.public) {
            Some(ip) => format!("http://{ip}:{}", self.port),
            None => format!("http://127.0.0.1:{}", self.port),
        }
    }

    /// True when `--public` found no address, so a phone cannot follow a code from here.
    #[must_use]
    pub const fn stuck_on_this_machine(&self) -> bool {
        self.public && self.chosen.is_none() && self.domain.is_empty() && self.lan.is_none()
    }
}

/// Untracked root file written by `up` and read by family builds.
pub const DEPLOYMENT_FILE: &str = "deployment.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deployment {
    pub server: String,
    pub bound: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Web,
    Android,
    Apple,
    Desktop,
}

impl Family {
    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        match word {
            "web" => Some(Self::Web),
            "android" => Some(Self::Android),
            "apple" => Some(Self::Apple),
            "desktop" => Some(Self::Desktop),
            _ => None,
        }
    }

    #[must_use]
    pub const fn requires_reachable_host(self) -> bool {
        matches!(self, Self::Android | Self::Apple)
    }
}

#[must_use]
pub fn host_of(address: &str) -> &str {
    let lower_https = address
        .get(..8)
        .is_some_and(|p| p.eq_ignore_ascii_case("https://"));
    let lower_http = address
        .get(..7)
        .is_some_and(|p| p.eq_ignore_ascii_case("http://"));
    let rest = if lower_https {
        &address[8..]
    } else if lower_http {
        &address[7..]
    } else {
        address
    };
    let authority = rest.split('/').next().unwrap_or(rest);
    if let Some(after_open) = authority.strip_prefix('[') {
        return after_open.split(']').next().unwrap_or(after_open);
    }
    if authority.matches(':').count() == 1 {
        authority.split(':').next().unwrap_or(authority)
    } else {
        authority
    }
}

#[must_use]
pub fn is_loopback_or_unspecified(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return ip.is_loopback() || ip.is_unspecified();
    }
    false
}

#[must_use]
pub fn own_network(host: &str) -> bool {
    let lower = host.to_ascii_lowercase();
    if lower == "localhost" || lower.ends_with(".local") {
        return true;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(v4) => is_own_v4(v4),
            IpAddr::V6(v6) => is_own_v6(v6),
        };
    }
    false
}

const fn is_own_v4(ip: Ipv4Addr) -> bool {
    ip.is_loopback() || ip.is_private() || ip.is_link_local()
}

const fn is_own_v6(ip: Ipv6Addr) -> bool {
    ip.is_loopback() || ip.is_unique_local() || (ip.segments()[0] & 0xffc0) == 0xfe80
}

/// Validate a server address for binding into a family build.
///
/// # Errors
/// Returns an error if the scheme/host is invalid, uses plain HTTP outside private networks, or
/// targets a loopback/unspecified address for a device family.
pub fn check_bind(server: &str, family: Family) -> Result<String> {
    let cleaned = checked(server)?;
    let host = host_of(&cleaned);
    if host.is_empty() {
        bail!("server address has no host: {cleaned}");
    }
    let is_http = cleaned
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"));
    if is_http && !own_network(host) {
        bail!(
            "plain http ({cleaned}) may reach only your own machine or network; use https:// for \
             a public deployment"
        );
    }
    if family.requires_reachable_host() && is_loopback_or_unspecified(host) {
        bail!(
            "{cleaned} only reaches this machine, so another device cannot reach it. Run \
             ./up --public, pass --site-address https://..., or pass --server <url>."
        );
    }
    Ok(cleaned)
}

#[must_use]
pub fn utc_date_today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    utc_date_from_unix_secs(secs)
}

#[must_use]
pub fn utc_date_from_unix_secs(secs: u64) -> String {
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(m <= 2);
    format!("{year:04}-{m:02}-{d:02}")
}

/// Write `deployment.json` at the repository root.
///
/// # Errors
/// Fails if serialization or writing fails.
pub fn write_deployment(root: &Path, server: &str) -> Result<PathBuf> {
    let path = root.join(DEPLOYMENT_FILE);
    let entry = Deployment {
        server: checked(server)?,
        bound: utc_date_today(),
    };
    let mut text = serde_json::to_string_pretty(&entry)?;
    text.push('\n');
    std::fs::write(&path, text).with_context(|| format!("could not write {}", path.display()))?;
    Ok(path)
}

/// Read `deployment.json` from the repository root.
///
/// # Errors
/// Fails if missing, unreadable, malformed, or holding an invalid address.
pub fn read_deployment(root: &Path) -> Result<Deployment> {
    let path = root.join(DEPLOYMENT_FILE);
    let text = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "{} was not found; run ./up --public first or pass --server <url>",
            path.display()
        )
    })?;
    let mut parsed: Deployment = serde_json::from_str(&text)
        .with_context(|| format!("{} is not valid deployment JSON", path.display()))?;
    parsed.server = checked(&parsed.server)?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAN: Option<IpAddr> = Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)));
    const PLAIN: Site<'static> = Site {
        chosen: None,
        domain: "",
        public: false,
        port: 8091,
        lan: LAN,
    };

    #[test]
    fn without_public_the_address_stays_on_this_machine() {
        assert_eq!(PLAIN.address(), "http://127.0.0.1:8091");
        assert!(!PLAIN.stuck_on_this_machine());
    }

    #[test]
    fn public_with_no_domain_uses_the_network_address() {
        let site = Site {
            public: true,
            ..PLAIN
        };
        assert_eq!(site.address(), "http://192.168.1.20:8091");
    }

    #[test]
    fn public_with_no_network_says_so() {
        let site = Site {
            public: true,
            lan: None,
            ..PLAIN
        };
        assert_eq!(site.address(), "http://127.0.0.1:8091");
        assert!(site.stuck_on_this_machine());
    }

    #[test]
    fn a_domain_wins_over_the_network_and_a_named_address_wins_over_both() {
        let branded = Site {
            domain: "watch.example",
            public: true,
            ..PLAIN
        };
        assert_eq!(branded.address(), "https://watch.example");
        let named = Site {
            chosen: Some("https://tunnel.example"),
            ..branded
        };
        assert_eq!(named.address(), "https://tunnel.example");
        assert!(!named.stuck_on_this_machine());
    }

    #[test]
    fn a_named_address_must_be_whole() {
        assert_eq!(
            checked("https://tunnel.example/").unwrap(),
            "https://tunnel.example"
        );
        assert_eq!(
            checked("http://192.168.1.20:8091").unwrap(),
            "http://192.168.1.20:8091"
        );
        for bad in ["tunnel.example", "https://", "ftp://tunnel.example", ""] {
            assert!(checked(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn this_machine_finds_a_route_or_says_it_has_none() {
        if let Some(ip) = lan_ip() {
            assert!(!ip.is_loopback() && !ip.is_unspecified(), "{ip}");
        }
    }

    #[test]
    fn bind_refuses_loopback_on_device_families_and_plain_public_http() {
        for loopback in [
            "http://127.0.0.1:8080",
            "http://localhost:8080",
            "http://[::1]:8080",
        ] {
            assert!(check_bind(loopback, Family::Android).is_err(), "{loopback}");
            assert!(check_bind(loopback, Family::Apple).is_err(), "{loopback}");
            assert!(check_bind(loopback, Family::Desktop).is_ok(), "{loopback}");
            assert!(check_bind(loopback, Family::Web).is_ok(), "{loopback}");
        }
        assert!(check_bind("http://0.0.0.0:8080", Family::Android).is_err());
        assert!(check_bind("http://0.0.0.0:8080", Family::Desktop).is_err());
        assert!(check_bind("http://streams.example", Family::Android).is_err());
        assert!(check_bind("http://streams.example", Family::Desktop).is_err());
        assert_eq!(
            check_bind("http://192.168.1.20:8080/", Family::Android).unwrap(),
            "http://192.168.1.20:8080"
        );
        assert_eq!(
            check_bind("https://watch.example/", Family::Apple).unwrap(),
            "https://watch.example"
        );
    }

    #[test]
    fn deployment_round_trips_and_formats_utc_dates() {
        assert_eq!(utc_date_from_unix_secs(0), "1970-01-01");
        assert_eq!(utc_date_from_unix_secs(1_794_826_800), "2026-11-16");
        let dir = std::env::temp_dir().join(format!("launcher-deploy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = write_deployment(&dir, "http://192.168.1.20:8080/").unwrap();
        assert_eq!(file, dir.join(DEPLOYMENT_FILE));
        let loaded = read_deployment(&dir).unwrap();
        assert_eq!(loaded.server, "http://192.168.1.20:8080");
        assert_eq!(loaded.bound.len(), 10);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
