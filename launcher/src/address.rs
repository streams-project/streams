//! The address a phone follows from a code on a television. The server sends it to the device, so
//! it must be one another device can reach.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use anyhow::{Result, bail};

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
}
