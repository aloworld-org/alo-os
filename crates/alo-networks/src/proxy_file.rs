//! The machine's proxy file, and the proxy a person hands the broker to put in it.
//!
//! `docs/features.md` v0.5 promises a proxy **machine-wide**, and `alo-proxy`
//! holds the one setting ([`alo_proxy::Kept`]) without reading or writing any
//! file. A setting that is machine-wide has to be written somewhere only the
//! machine can write and everybody can read, which is the broker's to do: this is
//! the shape of both files it touches, so the side that asks and the side that
//! writes agree on them byte for byte. `docs/contracts/machine-proxy-file.md`
//! is the contract.
//!
//! | file | written by | holds |
//! |---|---|---|
//! | [`THE_MACHINES_PROXY`] | the broker, as root, `0644` | an [`alo_proxy::Kept`] |
//! | [`THE_WANTED_PROXY`] | the person, in the broker's own directory for it | the [`alo_proxy::TheProxy`] they chose |
//!
//! # A proxy is carried to the broker by its identity, and nothing else
//!
//! The broker's door takes no text. So a person's choice in Settings is written
//! as [`wanted`] bytes where the broker looks, and the request that crosses the
//! door is the digest of exactly those bytes. The broker digests what it finds;
//! bytes changed after the approval are a different identity, and nothing is
//! set.
//!
//! # Nothing read from a file is believed until it is rebuilt
//!
//! `alo-proxy`'s types deserialise without re-running their checks, which is
//! right for a file only the machine writes and wrong for one a person's
//! programs can. So [`rechecked`] and [`kept_on_this_machine`] rebuild every
//! address, exception and configuration through `alo-proxy`'s own checked
//! constructors and refuse anything that does not come back identical — a host
//! with a space in it, a password name that is blank, an exception that is not
//! a host. What the broker writes is only ever what `alo-proxy` would itself
//! have made.

use alo_proxy::{
    ConfigurationAddress, Exceptions, Kept, ProxyAddress, SetBy, TheProxy, WhereThePasswordIs,
};

/// Where the machine's proxy is kept.
///
/// `/etc/alo-proxy`, which the broker's unit makes (`ConfigurationDirectory=`)
/// `0755` and root's: every road out reads it, and only the broker writes it.
pub const THE_MACHINES_PROXY: &str = "/etc/alo-proxy/proxy.json";

/// Where a person puts the proxy they chose, for the broker to set.
///
/// Inside the broker's runtime directory, in a folder the broker makes `0770`
/// in the person's group at start-up: the person can write there, the agent's
/// own login cannot reach it, and no component of the path is anybody's but
/// root's.
pub const THE_WANTED_PROXY: &str = "/run/alo-broker/wanted/proxy.json";

/// The most bytes either file may be. A proxy setting is a few hundred.
pub const LONGEST_FILE: u64 = 64 * 1024;

/// Why some bytes are not a proxy setting this machine will keep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAProxy(pub String);

impl std::fmt::Display for NotAProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a proxy setting this machine keeps: {}", self.0)
    }
}

impl std::error::Error for NotAProxy {}

/// The bytes a person's chosen proxy is handed to the broker as.
///
/// # Errors
/// [`NotAProxy`] if it cannot be written, which `alo-proxy`'s shapes cannot
/// cause.
pub fn wanted(proxy: &TheProxy) -> Result<Vec<u8>, NotAProxy> {
    serde_json::to_vec(proxy).map_err(|why| NotAProxy(why.to_string()))
}

/// The proxy a person handed the broker, rebuilt through `alo-proxy`'s checks.
///
/// # Errors
/// [`NotAProxy`] for anything that is not exactly a setting `alo-proxy` would
/// have made.
pub fn rechecked(bytes: &[u8]) -> Result<TheProxy, NotAProxy> {
    let read: TheProxy = serde_json::from_slice(bytes).map_err(|why| NotAProxy(why.to_string()))?;
    rebuilt(&read)
}

/// The bytes the machine's proxy file holds.
///
/// # Errors
/// [`NotAProxy`] if it cannot be written, which `alo-proxy`'s shapes cannot
/// cause.
pub fn machines(kept: &Kept) -> Result<Vec<u8>, NotAProxy> {
    let mut bytes = serde_json::to_vec(kept).map_err(|why| NotAProxy(why.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// What the machine's proxy file says, rebuilt through `alo-proxy`'s checks.
///
/// # Errors
/// [`NotAProxy`].
pub fn kept_on_this_machine(bytes: &[u8]) -> Result<Kept, NotAProxy> {
    let read: Kept = serde_json::from_slice(bytes).map_err(|why| NotAProxy(why.to_string()))?;
    let proxy = rebuilt(read.proxy())?;
    Ok(match read.set_by() {
        SetBy::AnOrganisation => Kept::by_an_organisation(proxy),
        SetBy::ThisPerson => Kept::by_this_person(proxy),
    })
}

/// The same setting, made again by `alo-proxy`'s own constructors — and
/// refused unless it comes back identical.
fn rebuilt(proxy: &TheProxy) -> Result<TheProxy, NotAProxy> {
    let again = match proxy {
        TheProxy::None => TheProxy::None,
        TheProxy::Manual {
            http,
            https,
            exceptions,
        } => TheProxy::Manual {
            http: http.as_ref().map(address).transpose()?,
            https: https.as_ref().map(address).transpose()?,
            exceptions: Exceptions::of(exceptions.each())
                .map_err(|why| NotAProxy(format!("an exception: {why:?}")))?,
        },
        TheProxy::Automatic { at } => TheProxy::Automatic {
            at: ConfigurationAddress::checked(at.as_str())
                .map_err(|why| NotAProxy(format!("a configuration address: {why:?}")))?,
        },
    };
    if &again == proxy {
        Ok(again)
    } else {
        Err(NotAProxy(
            "a setting that does not read back as alo-proxy writes it".to_owned(),
        ))
    }
}

/// One address, made again.
fn address(written: &ProxyAddress) -> Result<ProxyAddress, NotAProxy> {
    let refused = |why: &dyn std::fmt::Debug| NotAProxy(format!("an address: {why:?}"));
    let checked = ProxyAddress::checked(written.spoken_to(), written.host(), written.port())
        .map_err(|why| refused(&why))?;
    match (written.name(), written.password()) {
        (None, None) => Ok(checked),
        (Some(name), Some(password)) => {
            let password =
                WhereThePasswordIs::named(password.as_str()).map_err(|why| refused(&why))?;
            checked
                .signing_in(name, password)
                .map_err(|why| refused(&why))
        }
        _ => Err(NotAProxy(
            "an address with a name and nowhere for its password, or the other way round"
                .to_owned(),
        )),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_proxy::SpokenTo;

    /// A company's proxy, with a name and where its password is kept.
    fn the_company_proxy() -> TheProxy {
        TheProxy::one(
            ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 3128)
                .unwrap()
                .signing_in(
                    "anna",
                    WhereThePasswordIs::named("the company proxy").unwrap(),
                )
                .unwrap(),
        )
        .excepting(Exceptions::of(["intranet.example.com"]).unwrap())
    }

    /// **What a person chose reads back as exactly that**, and so does what the
    /// machine keeps, whoever set it.
    #[test]
    fn every_shape_reads_back_as_it_was_written() {
        for proxy in [
            TheProxy::None,
            the_company_proxy(),
            TheProxy::Automatic {
                at: ConfigurationAddress::checked("https://wpad.example.com/proxy.pac").unwrap(),
            },
        ] {
            assert_eq!(rechecked(&wanted(&proxy).unwrap()), Ok(proxy.clone()));
            for kept in [
                Kept::by_this_person(proxy.clone()),
                Kept::by_an_organisation(proxy.clone()),
            ] {
                assert_eq!(
                    kept_on_this_machine(&machines(&kept).unwrap()),
                    Ok(kept.clone())
                );
            }
        }
    }

    /// **Anything `alo-proxy` would not have made is refused**, although it
    /// deserialises: a host with a space in it, a password written into the
    /// host, a blank keyring name, a name with nowhere for its password, an
    /// exception that is not a host, a configuration address on two lines, and
    /// things that are not a setting at all.
    #[test]
    fn a_setting_alo_proxy_would_not_have_made_is_refused() {
        for written in [
            r#"{"proxy":"manual","http":{"spoken_to":"http","host":"proxy example.com","port":3128}}"#,
            r#"{"proxy":"manual","http":{"spoken_to":"http","host":"anna:pw@proxy.example.com","port":3128}}"#,
            r#"{"proxy":"manual","http":{"spoken_to":"http","host":"proxy.example.com","port":0}}"#,
            r#"{"proxy":"manual","http":{"spoken_to":"http","host":"proxy.example.com","port":3128,"name":"anna","password":" "}}"#,
            r#"{"proxy":"manual","http":{"spoken_to":"http","host":"proxy.example.com","port":3128,"name":"anna"}}"#,
            r#"{"proxy":"manual","exceptions":["not a host"]}"#,
            "{\"proxy\":\"automatic\",\"at\":\"https://wpad.example.com/\\nproxy.pac\"}",
        ] {
            // Each of these is a setting as far as serde is concerned, which is
            // the point: only rebuilding it refuses it.
            assert!(
                serde_json::from_str::<TheProxy>(written).is_ok(),
                "{written} did not even deserialise"
            );
            assert!(rechecked(written.as_bytes()).is_err(), "{written}");
        }
        for written in [r#"{"proxy":"everything"}"#, "rm -rf /", ""] {
            assert!(rechecked(written.as_bytes()).is_err(), "{written}");
        }
    }

    /// **The paths are where the contract says, and the contract's own example
    /// is a file this machine reads** — so the page a reader builds against and
    /// the code cannot drift apart.
    #[test]
    fn the_files_are_where_the_contract_says_and_its_example_reads() {
        let contract = include_str!("../../../docs/contracts/machine-proxy-file.md");
        assert!(contract.contains(&format!("`{THE_MACHINES_PROXY}`")));
        assert!(contract.contains(&format!("`{THE_WANTED_PROXY}`")));
        assert!(THE_WANTED_PROXY.starts_with("/run/alo-broker/"));
        let example = contract
            .lines()
            .find(|line| line.starts_with("{\"proxy\":"))
            .unwrap();
        let kept = kept_on_this_machine(example.as_bytes()).unwrap();
        assert_eq!(kept.set_by(), SetBy::ThisPerson);
        assert_eq!(
            machines(&kept).unwrap(),
            format!("{example}\n").into_bytes()
        );
    }
}
