//! Outbound guard shared by server and finder: resolve, pin the address, check each hop, cap bytes.
//!
//! One deny table, in one place, reached through the client's own resolver. That matters more than
//! it reads: a guard that resolved a name, judged the answer and then handed the name back to the
//! client would let a name answer publicly while it is judged and privately when the socket opens.
//! Here the resolver is the only thing that turns a name into addresses, so the socket can only be
//! opened on an address this crate returned, and every redirect hop goes through it again.

use std::collections::BTreeSet;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use reqwest::Url;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE, LOCATION};
use thiserror::Error;

/// How long a connection may take to open, whatever the whole call's deadline is.
const CONNECT_WITHIN: Duration = Duration::from_secs(3);

/// Why a call was refused. Callers turn this into their own "the service did not answer": the
/// reason stays in the log and never reaches a viewer.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum Refusal {
    /// The address is not one a caller may reach. The word is the range that refused it.
    #[error("the address is {0}, which is not a public service")]
    Address(&'static str),
    /// Not a web address at all, or one that cannot be parsed.
    #[error("not an address this guard calls")]
    NotAnAddress,
    /// The name resolved to nothing.
    #[error("the name resolved to no address")]
    Unresolved,
    /// More redirects than the profile allows, or a redirect on a call that may not be replayed.
    #[error("too many hops")]
    Hops,
    /// The answer passed a byte cap, before or after decoding.
    #[error("the answer is larger than the cap")]
    TooLarge,
    /// The whole call ran out of time.
    #[error("no answer within the deadline")]
    Deadline,
    /// Nothing answered.
    #[error("the service did not answer")]
    Unreachable,
    /// The answer arrived in an encoding this guard does not decode.
    #[error("the answer is encoded in a way this guard does not read")]
    Encoding,
    /// The client could not be built, which is a fault in this install and not in the answer.
    #[error("the outbound client could not be built")]
    Broken,
}

/// The range that refuses this address, or `None` when it is a public one.
///
/// One table for both families. An IPv6 address that carries an IPv4 address inside it is judged
/// on the address inside, so neither family is a way around the other.
#[must_use]
pub fn denied(ip: IpAddr) -> Option<&'static str> {
    match ip {
        IpAddr::V4(four) => denied_v4(four),
        IpAddr::V6(six) => denied_v6(six),
    }
}

fn denied_v4(ip: Ipv4Addr) -> Option<&'static str> {
    let [a, b, c, _] = ip.octets();
    Some(match () {
        () if ip.is_unspecified() => "unspecified",
        () if ip.is_loopback() => "loopback",
        () if ip.is_private() => "private",
        () if ip.is_link_local() => "link-local",
        // 100.64.0.0/10, the range a carrier puts between its customers and the internet.
        () if a == 100 && (64..128).contains(&b) => "carrier shared",
        // 192.0.0.0/24, held for protocols rather than hosts.
        () if a == 192 && b == 0 && c == 0 => "protocol reserved",
        () if ip.is_documentation() => "documentation",
        // 198.18.0.0/15, for testing equipment.
        () if a == 198 && (b == 18 || b == 19) => "benchmarking",
        () if ip.is_multicast() => "multicast",
        () if ip.is_broadcast() => "broadcast",
        // 240.0.0.0/4, never assigned.
        () if a >= 240 => "reserved",
        () => return None,
    })
}

/// The IPv4 address an IPv6 one carries, if any. Mapped (`::ffff:a.b.c.d`), 6to4
/// (`2002:a.b.c.d::/48`), NAT64 (`64:ff9b::/96`) and the deprecated compatible form (`::a.b.c.d`)
/// all end at an IPv4 host, so each is handed to the IPv4 table.
fn inner_v4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let parts = ip.segments();
    let from = |high: u16, low: u16| {
        Ipv4Addr::new(
            (high >> 8) as u8,
            (high & 0xff) as u8,
            (low >> 8) as u8,
            (low & 0xff) as u8,
        )
    };
    if let Some(four) = ip.to_ipv4_mapped() {
        return Some(four);
    }
    if parts[0] == 0x2002 {
        return Some(from(parts[1], parts[2]));
    }
    if parts[0] == 0x0064 && parts[1] == 0xff9b && parts[2..6] == [0, 0, 0, 0] {
        return Some(from(parts[6], parts[7]));
    }
    // `::a.b.c.d`. `::` and `::1` are named by their own rows above this call, so anything left
    // here with a leading run of zeros is carrying an address.
    if parts[..6] == [0, 0, 0, 0, 0, 0] {
        return Some(from(parts[6], parts[7]));
    }
    None
}

fn denied_v6(ip: Ipv6Addr) -> Option<&'static str> {
    if ip.is_unspecified() {
        return Some("unspecified");
    }
    if ip.is_loopback() {
        return Some("loopback");
    }
    if let Some(four) = inner_v4(ip) {
        return denied_v4(four);
    }
    let parts = ip.segments();
    Some(match () {
        // fc00::/7, the family's own private range.
        () if parts[0] & 0xfe00 == 0xfc00 => "private",
        // fe80::/10.
        () if parts[0] & 0xffc0 == 0xfe80 => "link-local",
        () if ip.is_multicast() => "multicast",
        // 100::/64, which a router is told to drop.
        () if parts[0] == 0x0100 && parts[1..4] == [0, 0, 0] => "discard only",
        () if parts[0] == 0x2001 && parts[1] == 0x0db8 => "documentation",
        // 2001::/32, a tunnel over IPv4.
        () if parts[0] == 0x2001 && parts[1] == 0 => "tunnelled",
        () => return None,
    })
}

/// False when this address names a host inside this machine or this network.
///
/// For an address nothing here will fetch: a source address goes out to a device, which fetches it
/// on its own network, so the guard never sees it and the deny table has to be read against the
/// text instead. An address written as numbers is judged; a name is resolved somewhere else
/// entirely and cannot be judged from here, so it passes and the device's own network decides.
#[must_use]
pub fn public_address(url: &str) -> bool {
    let Ok(parsed) = Url::parse(url) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    match host.trim_matches(['[', ']']).parse::<IpAddr>() {
        Ok(literal) => denied(literal).is_none(),
        Err(_) => true,
    }
}

/// The addresses a name may be reached at, or why it may not be.
///
/// `configured` is true for an endpoint this install's own configuration names. A provider list or
/// a service address the operator ships is a different risk from a URL a caller supplies: the
/// operator's finder and subtitle packs listen on `127.0.0.1`, so a configured endpoint opts out
/// of the private-range rule and nothing else does. The allowance is for one host and one port
/// together, never for the rest of a host's ports.
fn allow(configured: bool, found: Vec<SocketAddr>) -> Result<Vec<SocketAddr>, Refusal> {
    if found.is_empty() {
        return Err(Refusal::Unresolved);
    }
    if configured {
        return Ok(found);
    }
    // Every answer must pass, not just the one that would be used. A name with one public and one
    // private address is a name trying both doors.
    for address in &found {
        if let Some(range) = denied(address.ip()) {
            return Err(Refusal::Address(range));
        }
    }
    Ok(found)
}

/// The deny table as the client's own resolver.
///
/// A resolver is handed a name and nothing else, so what it holds is the configured names. The
/// client it belongs to is picked on the host and the port, so every call that reaches this
/// resolver with a filled set was already matched to a configured endpoint.
struct Reach {
    operator: Arc<BTreeSet<String>>,
}

impl Resolve for Reach {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_ascii_lowercase();
        let configured = self.operator.contains(&host);
        Box::pin(async move {
            // Port 0: reqwest puts the address's own port back over it.
            let found: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0))
                .await
                .map_err(|_| Refusal::Unresolved)?
                .collect();
            Ok(Box::new(allow(configured, found)?.into_iter()) as Addrs)
        })
    }
}

/// How much of the world one call may use. The two named profiles below are the only two this
/// project has; a caller that wants other numbers says so field by field.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Redirects followed. Each hop is checked again, so this bounds the work, not the safety.
    pub hops: u8,
    /// Connecting, every hop and the body together.
    pub deadline: Duration,
    /// Bytes read off the wire.
    pub wire_bytes: usize,
    /// Bytes handed to the caller, which is what an encoded body decodes to.
    pub body_bytes: usize,
}

impl Limits {
    /// The fetch profile: a file this server reads itself, so it stays shallow.
    #[must_use]
    pub const fn fetch(body_bytes: usize) -> Self {
        Self {
            hops: 5,
            deadline: Duration::from_secs(20),
            wire_bytes: body_bytes,
            body_bytes,
        }
    }

    /// The provider profile: a source list, which may move about more before it settles.
    #[must_use]
    pub const fn provider(body_bytes: usize) -> Self {
        Self {
            hops: 10,
            deadline: Duration::from_secs(20),
            wire_bytes: body_bytes,
            body_bytes,
        }
    }

    /// The same profile with another deadline, for a caller that has a budget of its own.
    #[must_use]
    pub const fn within(mut self, deadline: Duration) -> Self {
        self.deadline = deadline;
        self
    }
}

/// One outbound call. It is built here and sent by [`Guard::fetch`], so there is no way to make a
/// request that skipped the caps.
#[derive(Clone, Debug)]
pub struct Ask {
    url: String,
    body: Option<Vec<u8>>,
    bearer: Option<String>,
    query: Vec<(String, String)>,
}

impl Ask {
    /// Read this address.
    #[must_use]
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            body: None,
            bearer: None,
            query: Vec::new(),
        }
    }

    /// Post this JSON. A redirect to a posted call is refused rather than replayed: sending a body
    /// on to an address the first answer named is how a call ends up somewhere its caller never
    /// chose.
    #[must_use]
    pub fn post_json(url: impl Into<String>, body: Vec<u8>) -> Self {
        Self {
            body: Some(body),
            ..Self::get(url)
        }
    }

    /// Carry a service token.
    #[must_use]
    pub fn bearer(mut self, token: &str) -> Self {
        self.bearer = Some(token.to_owned());
        self
    }

    /// Add one query value.
    #[must_use]
    pub fn query(mut self, name: &str, value: impl Into<String>) -> Self {
        self.query.push((name.to_owned(), value.into()));
        self
    }
}

/// What one call returned, within every cap.
#[derive(Clone, Debug)]
pub struct Fetched {
    pub status: u16,
    pub bytes: Vec<u8>,
}

impl Fetched {
    /// True when the service answered with a success.
    #[must_use]
    pub const fn ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
}

/// The only way out of this process for the code that holds one.
///
/// Two clients, because a resolver is only ever asked about a name and never about a port: the
/// operator's own endpoints are called through the client whose resolver may answer with a private
/// address, everything else through the client whose resolver reads the deny table over every
/// answer. Which one a call uses is decided on the host and the port together, so an allowance
/// cannot widen to the rest of a host between the check and the socket.
#[derive(Clone, Debug)]
pub struct Guard {
    /// For the configured endpoints, whose packs listen on a private address.
    operator_client: reqwest::Client,
    /// For every other address, including a hop or a locator a caller handed over.
    public_client: reqwest::Client,
    limits: Limits,
    /// This install's own endpoints, each as `host:port`.
    operator: Arc<BTreeSet<String>>,
}

/// One endpoint, written the one way: the host folded to lower case, its brackets off, and the
/// port the scheme would use when the address does not name one.
fn endpoint(host: &str, port: u16) -> String {
    format!(
        "{}:{port}",
        host.trim_matches(['[', ']']).to_ascii_lowercase()
    )
}

/// A configured address as an endpoint, so a caller hands over the addresses it already has rather
/// than picking hosts and ports out of them itself.
///
/// The port is part of the key because the allowance is for one pack and not for a machine: a pack
/// configured on `127.0.0.1:41234` used to open every other port of `127.0.0.1` as well, the
/// database and this server's own among them, to any caller whose URL or redirect named one.
fn endpoint_of(address: &str) -> Option<String> {
    let url = Url::parse(address).ok()?;
    Some(endpoint(url.host_str()?, url.port_or_known_default()?))
}

/// The name of a configured address, when it is a name. An address written as numbers never
/// reaches a resolver, so it is left out of the set the resolver reads.
fn name_of(address: &str) -> Option<String> {
    let host = Url::parse(address).ok()?.host_str()?.to_ascii_lowercase();
    let bare = host.trim_matches(['[', ']']);
    bare.parse::<IpAddr>().err().map(|_| host)
}

impl Guard {
    /// A guard for these limits over these operator hosts.
    ///
    /// Both arguments are required on purpose. A guard built with no limits, or one that quietly
    /// trusted every host, would be the hole this crate exists to close, so there is no default
    /// that means either.
    ///
    /// # Errors
    /// The HTTP client could not be built.
    pub fn new(limits: Limits, operator_hosts: &[String]) -> Result<Self, Refusal> {
        let operator: Arc<BTreeSet<String>> = Arc::new(
            operator_hosts
                .iter()
                .filter_map(|a| endpoint_of(a))
                .collect(),
        );
        // The resolver cannot see a port, so what it keys on is the configured names. It is only
        // ever reached by a call this guard already matched to a configured endpoint.
        let names: Arc<BTreeSet<String>> =
            Arc::new(operator_hosts.iter().filter_map(|a| name_of(a)).collect());
        Ok(Self {
            operator_client: client(names)?,
            public_client: client(Arc::new(BTreeSet::new()))?,
            limits,
            operator,
        })
    }

    /// True when this address is one of the configured endpoints, host and port both.
    fn configured(&self, url: &Url) -> bool {
        let Some(host) = url.host_str() else {
            return false;
        };
        let Some(port) = url.port_or_known_default() else {
            return false;
        };
        self.operator.contains(&endpoint(host, port))
    }

    /// Make this call, within every cap.
    ///
    /// # Errors
    /// See [`Refusal`].
    pub async fn fetch(&self, ask: &Ask) -> Result<Fetched, Refusal> {
        self.fetch_with(ask, self.limits).await
    }

    /// The same, with other caps for this one call. The deny table belongs to the client and so
    /// cannot be given away here; only the numbers move, which is why a caller that reads two
    /// sizes of thing needs one guard and not two.
    ///
    /// # Errors
    /// See [`Refusal`].
    pub async fn fetch_with(&self, ask: &Ask, limits: Limits) -> Result<Fetched, Refusal> {
        // One deadline over connecting, every hop and the body, so a slow body cannot buy time a
        // per-hop timeout would have given it again.
        tokio::time::timeout(limits.deadline, self.hops(ask, limits))
            .await
            .map_err(|_| Refusal::Deadline)?
    }

    async fn hops(&self, ask: &Ask, limits: Limits) -> Result<Fetched, Refusal> {
        let mut url = Url::parse(&ask.url).map_err(|_| Refusal::NotAnAddress)?;
        for _ in 0..=limits.hops {
            self.checked(&url)?;
            let answer = self.one(&url, ask).await?;
            let status = answer.status().as_u16();
            let moved = (300..400).contains(&status);
            let next = answer
                .headers()
                .get(LOCATION)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned);
            let Some(next) = moved.then_some(next).flatten() else {
                let bytes = self.read(answer, limits).await?;
                return Ok(Fetched { status, bytes });
            };
            if ask.body.is_some() {
                return Err(Refusal::Hops);
            }
            url = url.join(&next).map_err(|_| Refusal::NotAnAddress)?;
        }
        Err(Refusal::Hops)
    }

    /// What can be judged about a hop before it is called. The resolver has the last word, because
    /// it is what opens the socket; this says the same thing earlier and with the range named, and
    /// it is the whole check for an address written as numbers, which no resolver sees.
    fn checked(&self, url: &Url) -> Result<(), Refusal> {
        if !matches!(url.scheme(), "http" | "https") {
            return Err(Refusal::NotAnAddress);
        }
        let Some(host) = url.host_str() else {
            return Err(Refusal::NotAnAddress);
        };
        let configured = self.configured(url);
        if !configured
            && let Ok(literal) = host.trim_matches(['[', ']']).parse::<IpAddr>()
            && let Some(range) = denied(literal)
        {
            return Err(Refusal::Address(range));
        }
        Ok(())
    }

    async fn one(&self, url: &Url, ask: &Ask) -> Result<reqwest::Response, Refusal> {
        // The allowance and the socket are chosen by the same rule, so a call that was judged as
        // a caller's URL cannot be opened by the resolver that may answer privately.
        let client = if self.configured(url) {
            &self.operator_client
        } else {
            &self.public_client
        };
        let mut call = match &ask.body {
            Some(body) => client
                .post(url.clone())
                .header(CONTENT_TYPE, "application/json")
                .body(body.clone()),
            None => client.get(url.clone()),
        };
        if !ask.query.is_empty() {
            call = call.query(&ask.query);
        }
        if let Some(token) = &ask.bearer {
            call = call.bearer_auth(token);
        }
        // Nothing here wants an encoded body, so say so. One that arrives anyway is decoded below
        // under the cap rather than by a decompressor with no cap at all.
        call = call.header(ACCEPT_ENCODING, "identity");
        call.send().await.map_err(refusal_of)
    }

    /// Read the body, stopping at the cap. Nothing is buffered whole first, so a body larger than
    /// the cap costs the cap and not its own size.
    async fn read(
        &self,
        mut answer: reqwest::Response,
        limits: Limits,
    ) -> Result<Vec<u8>, Refusal> {
        let coded = answer
            .headers()
            .get(CONTENT_ENCODING)
            .and_then(|value| value.to_str().ok())
            .map(str::to_ascii_lowercase);
        // A declared length over the cap is refused before a byte is read.
        if answer
            .content_length()
            .is_some_and(|length| length > limits.wire_bytes as u64)
        {
            return Err(Refusal::TooLarge);
        }
        let mut wire: Vec<u8> = Vec::new();
        while let Some(chunk) = answer.chunk().await.map_err(refusal_of)? {
            if wire.len() + chunk.len() > limits.wire_bytes {
                return Err(Refusal::TooLarge);
            }
            wire.extend_from_slice(&chunk);
        }
        match coded.as_deref() {
            None | Some("") | Some("identity") => {
                if wire.len() > limits.body_bytes {
                    return Err(Refusal::TooLarge);
                }
                Ok(wire)
            }
            Some("gzip" | "x-gzip") => inflate(&wire, limits.body_bytes, true),
            Some("deflate") => inflate(&wire, limits.body_bytes, false),
            Some(_) => Err(Refusal::Encoding),
        }
    }
}

/// One HTTP client whose only way of turning a name into an address is this crate's resolver.
fn client(names: Arc<BTreeSet<String>>) -> Result<reqwest::Client, Refusal> {
    reqwest::Client::builder()
        .connect_timeout(CONNECT_WITHIN)
        // Hops are followed by the guard, one at a time, so each one is resolved and checked again.
        .redirect(reqwest::redirect::Policy::none())
        .dns_resolver(Arc::new(Reach { operator: names }))
        .build()
        .map_err(|_| Refusal::Broken)
}

/// Decode under the cap. The reader stops one byte past it, so a small body that claims to expand
/// to gigabytes costs the cap and the rest is never decoded.
fn inflate(wire: &[u8], cap: usize, gzip: bool) -> Result<Vec<u8>, Refusal> {
    let reader: Box<dyn Read> = if gzip {
        Box::new(flate2::read::GzDecoder::new(wire))
    } else {
        Box::new(flate2::read::ZlibDecoder::new(wire))
    };
    let mut out = Vec::new();
    reader
        .take(cap as u64 + 1)
        .read_to_end(&mut out)
        .map_err(|_| Refusal::Encoding)?;
    if out.len() > cap {
        return Err(Refusal::TooLarge);
    }
    Ok(out)
}

/// Why this call failed. A refusal the resolver raised is carried back out through the error
/// chain, so a caller is told the address was the problem and not that the host was down.
fn refusal_of(error: reqwest::Error) -> Refusal {
    if error.is_timeout() {
        return Refusal::Deadline;
    }
    let mut step: Option<&(dyn std::error::Error + 'static)> = Some(&error);
    while let Some(cause) = step {
        if let Some(found) = cause.downcast_ref::<Refusal>() {
            return found.clone();
        }
        step = cause.source();
    }
    Refusal::Unreachable
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    const LOOPBACK: &str = "http://127.0.0.1";

    fn guard(limits: Limits, port: u16) -> Guard {
        // The test server listens on loopback, which is exactly what an operator's own finder and
        // subtitle packs do, so it goes in as a configured host.
        Guard::new(limits, &[format!("{LOOPBACK}:{port}")]).expect("a guard")
    }

    fn small() -> Limits {
        Limits::fetch(64).within(Duration::from_secs(2))
    }

    async fn listening() -> (tokio::net::TcpListener, u16) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a port");
        let port = listener.local_addr().expect("an address").port();
        (listener, port)
    }

    /// Read the request head and throw it away. The tests are about the answer.
    async fn head(stream: &mut tokio::net::TcpStream) {
        use tokio::io::AsyncReadExt;
        let mut seen = Vec::new();
        let mut byte = [0u8; 1];
        while seen.len() < 8192 {
            if stream.read(&mut byte).await.unwrap_or(0) != 1 {
                return;
            }
            seen.push(byte[0]);
            if seen.ends_with(b"\r\n\r\n") {
                return;
            }
        }
    }

    /// A server that answers every request with the same bytes. Raw TCP, because half of what is
    /// tested here is how a body arrives rather than what it holds.
    async fn serving(reply: impl Fn(u16) -> Vec<u8> + Send + 'static) -> u16 {
        let (listener, port) = listening().await;
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            while let Ok((mut stream, _)) = listener.accept().await {
                head(&mut stream).await;
                let _ = stream.write_all(&reply(port)).await;
                let _ = stream.flush().await;
            }
        });
        port
    }

    /// A server that promises a body and then stops writing, so only a deadline ends the call.
    async fn stalling() -> u16 {
        let (listener, port) = listening().await;
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            while let Ok((mut stream, _)) = listener.accept().await {
                head(&mut stream).await;
                let _ = stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 32\r\n\r\na")
                    .await;
                let _ = stream.flush().await;
                // Held open for longer than any deadline in these tests.
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
        port
    }

    fn answer(body: &[u8], headers: &str) -> Vec<u8> {
        let mut out = format!("HTTP/1.1 200 OK\r\n{headers}\r\n").into_bytes();
        out.extend_from_slice(body);
        out
    }

    #[test]
    fn h72_every_closed_range_is_closed_in_both_families() {
        for text in [
            "0.0.0.0",
            "127.0.0.1",
            "127.1.2.3",
            "10.0.0.1",
            "172.16.4.4",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "192.0.0.1",
            "192.0.2.5",
            "198.51.100.5",
            "203.0.113.5",
            "198.18.0.1",
            "224.0.0.1",
            "255.255.255.255",
            "240.0.0.1",
            "::",
            "::1",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "ff02::1",
            "100::1",
            "2001:db8::1",
            "2001::1",
        ] {
            let ip: IpAddr = text.parse().expect("an address");
            assert!(denied(ip).is_some(), "{text} is not refused");
        }
        for text in ["93.184.216.34", "8.8.8.8", "172.32.0.1", "2606:4700::1111"] {
            let ip: IpAddr = text.parse().expect("an address");
            assert_eq!(denied(ip), None, "{text} is refused");
        }
    }

    #[test]
    fn h73_an_ipv4_address_carried_inside_an_ipv6_one_is_judged_the_same() {
        let judged = |text: &str| denied(text.parse::<IpAddr>().expect("an address"));
        // Mapped, 6to4, NAT64 and the compatible form, each over 127.0.0.1 and over a public host.
        assert_eq!(judged("::ffff:127.0.0.1"), Some("loopback"));
        assert_eq!(judged("::ffff:169.254.169.254"), Some("link-local"));
        assert_eq!(judged("2002:7f00:0001::"), Some("loopback"));
        assert_eq!(judged("64:ff9b::10.0.0.1"), Some("private"));
        assert_eq!(judged("::10.0.0.1"), Some("private"));
        assert_eq!(judged("::ffff:93.184.216.34"), None);
    }

    #[test]
    fn s4_a_configured_host_may_be_private_and_nothing_else_may() {
        let private: Vec<SocketAddr> = vec!["127.0.0.1:9000".parse().expect("an address")];
        assert!(allow(true, private.clone()).is_ok(), "the operator's own");
        assert_eq!(
            allow(false, private),
            Err(Refusal::Address("loopback")),
            "a caller's URL"
        );
        assert_eq!(allow(false, Vec::new()), Err(Refusal::Unresolved));
    }

    #[test]
    fn h10_a_name_with_one_private_answer_is_refused_whole() {
        // Answering with both doors open is the whole trick, so the public answer does not save it.
        let both: Vec<SocketAddr> = vec![
            "93.184.216.34:443".parse().expect("an address"),
            "169.254.169.254:443".parse().expect("an address"),
        ];
        assert_eq!(allow(false, both), Err(Refusal::Address("link-local")));
    }

    #[tokio::test]
    async fn h18_a_hop_from_a_public_answer_to_a_private_address_is_refused() {
        let port = serving(|_| {
            b"HTTP/1.1 302 Found\r\nLocation: http://10.0.0.1/next\r\nContent-Length: 0\r\n\r\n"
                .to_vec()
        })
        .await;
        let found = guard(small(), port)
            .fetch(&Ask::get(format!("{LOOPBACK}:{port}/start")))
            .await;
        assert_eq!(found.err(), Some(Refusal::Address("private")));
    }

    #[tokio::test]
    async fn h18_a_hop_that_comes_back_to_itself_ends_at_the_cap() {
        let port = serving(|port| {
            format!(
                "HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:{port}/again\r\n\
                 Content-Length: 0\r\n\r\n"
            )
            .into_bytes()
        })
        .await;
        let limits = Limits { hops: 2, ..small() };
        let found = guard(limits, port)
            .fetch(&Ask::get(format!("{LOOPBACK}:{port}/start")))
            .await;
        assert_eq!(found.err(), Some(Refusal::Hops));
    }

    #[tokio::test]
    async fn h74_a_body_over_the_cap_is_refused_whether_or_not_it_declares_its_length() {
        let declared = serving(|_| answer(&[b'x'; 4096], "Content-Length: 4096\r\n")).await;
        let found = guard(small(), declared)
            .fetch(&Ask::get(format!("{LOOPBACK}:{declared}/big")))
            .await;
        assert_eq!(found.err(), Some(Refusal::TooLarge), "declared");

        // No length and the connection closed at the end, so the cap is the only thing counting.
        let streamed = serving(|_| answer(&[b'x'; 4096], "Connection: close\r\n")).await;
        let found = guard(small(), streamed)
            .fetch(&Ask::get(format!("{LOOPBACK}:{streamed}/big")))
            .await;
        assert_eq!(found.err(), Some(Refusal::TooLarge), "streamed");
    }

    #[tokio::test]
    async fn h75_a_small_body_that_expands_past_the_cap_is_refused() {
        let mut packer = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
        packer.write_all(&vec![0u8; 4_000_000]).expect("packed");
        let packed = packer.finish().expect("packed");
        // The wire cap lets the packed bytes through, so what refuses it is the decoded cap and
        // nothing else: a body that arrives small and opens into four megabytes.
        let limits = Limits {
            wire_bytes: 64_000,
            body_bytes: 64,
            ..small()
        };
        assert!(
            packed.len() < limits.wire_bytes,
            "the bomb fits on the wire"
        );
        let port = serving(move |_| {
            answer(
                &packed,
                &format!(
                    "Content-Encoding: gzip\r\nContent-Length: {}\r\n",
                    packed.len()
                ),
            )
        })
        .await;
        let found = guard(limits, port)
            .fetch(&Ask::get(format!("{LOOPBACK}:{port}/bomb")))
            .await;
        assert_eq!(found.err(), Some(Refusal::TooLarge));
    }

    #[tokio::test]
    async fn h74_a_body_that_never_ends_is_cut_at_the_deadline() {
        let port = stalling().await;
        let limits = Limits::fetch(4096).within(Duration::from_millis(300));
        let began = std::time::Instant::now();
        let found = guard(limits, port)
            .fetch(&Ask::get(format!("{LOOPBACK}:{port}/slow")))
            .await;
        assert_eq!(found.err(), Some(Refusal::Deadline));
        assert!(began.elapsed() < Duration::from_secs(3), "it waited");
    }

    #[tokio::test]
    async fn s4_one_configured_pack_does_not_open_the_other_ports_of_its_host() {
        // Another service on this machine, answering happily to anything that reaches it.
        let other = serving(|_| answer(b"secret", "Content-Length: 6\r\n")).await;
        // The pack, which hands back a redirect to it. A pack is configured, what it says is not.
        let pack = serving(move |_| {
            format!(
                "HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:{other}/\r\n\
                 Content-Length: 0\r\n\r\n"
            )
            .into_bytes()
        })
        .await;
        let found = guard(small(), pack)
            .fetch(&Ask::get(format!("{LOOPBACK}:{pack}/v1/search")))
            .await;
        assert_eq!(
            found.err(),
            Some(Refusal::Address("loopback")),
            "a configured port opened the rest of its host"
        );
    }

    #[tokio::test]
    async fn s4_a_name_that_resolves_to_a_private_address_is_refused_at_connect_time() {
        let port = serving(|_| answer(b"secret", "Content-Length: 6\r\n")).await;
        // `localhost` is a name, so nothing the hop check can read refuses it: it passes the check
        // and is refused by the resolver, on the address the socket would have been opened on.
        // The configured endpoint is the same port written as numbers, which is not this host.
        let found = guard(small(), port)
            .fetch(&Ask::get(format!("http://localhost:{port}/v1/search")))
            .await;
        assert_eq!(found.err(), Some(Refusal::Address("loopback")));
        // And the same name is still refused when it arrives as a redirect from a configured pack.
        let pack = serving(move |_| {
            format!(
                "HTTP/1.1 302 Found\r\nLocation: http://localhost:{port}/\r\n\
                 Content-Length: 0\r\n\r\n"
            )
            .into_bytes()
        })
        .await;
        let found = guard(small(), pack)
            .fetch(&Ask::get(format!("{LOOPBACK}:{pack}/v1/search")))
            .await;
        assert_eq!(found.err(), Some(Refusal::Address("loopback")));
    }

    #[test]
    fn a_configured_address_is_keyed_on_its_host_and_its_port() {
        assert_eq!(
            endpoint_of("http://127.0.0.1:41234/"),
            Some(String::from("127.0.0.1:41234"))
        );
        // A scheme's own port is filled in, so the same endpoint written either way is one key.
        assert_eq!(
            endpoint_of("https://Packs.Example/v1"),
            Some(String::from("packs.example:443"))
        );
        assert_eq!(
            endpoint_of("http://[::1]:9000"),
            Some(String::from("::1:9000"))
        );
        // Only a name goes to the resolver; an address written as numbers never reaches one.
        assert_eq!(
            name_of("https://Packs.Example/v1"),
            Some(String::from("packs.example"))
        );
        assert_eq!(name_of("http://127.0.0.1:41234"), None);
        assert_eq!(name_of("http://[::1]:9000"), None);
    }

    #[tokio::test]
    async fn an_answer_within_every_cap_comes_back_whole() {
        let port = serving(|_| answer(b"{\"files\":[]}", "Content-Length: 12\r\n")).await;
        let found = guard(small(), port)
            .fetch(&Ask::get(format!("{LOOPBACK}:{port}/v1/search")).bearer("token"))
            .await
            .expect("an answer");
        assert!(found.ok());
        assert_eq!(found.bytes, b"{\"files\":[]}");
    }
}
