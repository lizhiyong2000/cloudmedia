/// Generate a URI domain from an domain name.
/// ```rust
/// # #[macro_use]
/// # extern crate cloudmedia;
/// # fn main() {
/// let domain = domain!("example.com");
/// let domain = domain!("example.com", 5060);
/// # }
/// ```
#[macro_export]
macro_rules! domain {
    ($domain:tt) => {
        cloudmedia::protocol::sip::Domain::Domain($domain.into(), None)
    };
    ($domain:tt, $port:tt) => {
        cloudmedia::protocol::sip::Domain::Domain($domain.into(), Some($port))
    };
}

/// Generate a URI domain from an ip address.
/// ```rust
/// # #[macro_use]
/// # extern crate cloudmedia;
/// # fn main() {
/// let domain = ip_domain!(192, 168, 0, 1);
/// let domain = ip_domain!(192, 168, 0, 1, 5060);
/// # }
/// ```
#[macro_export]
macro_rules! ip_domain {
    ($a:tt, $b:tt, $c:tt, $d:tt) => {
        cloudmedia::protocol::sip::Domain::Ipv4(::std::net::Ipv4Addr::new($a, $b, $c, $d), None)
    };

    ($a:tt, $b:tt, $c:tt, $d:tt, $port:tt) => {
        cloudmedia::protocol::sip::Domain::Ipv4(::std::net::Ipv4Addr::new($a, $b, $c, $d), Some($port))
    };
}

/// Generate a URI authentication from credentials.
/// ```rust
/// # #[macro_use]
/// # extern crate cloudmedia;
/// # fn main() {
/// let auth = uri_auth!("user");
/// let auth = uri_auth!("user", "pass");
/// # }
/// ```
#[macro_export]
macro_rules! uri_auth {
    ($u:tt) => {
        cloudmedia::protocol::sip::UriAuth::new($u)
    };
    ($u:tt, $p:tt) => {
        cloudmedia::protocol::sip::UriAuth::new($u).password($p)
    };
}

/// Generate `NamedHeader` from a uri;
/// ```rust
/// # #[macro_use]
/// # extern crate cloudmedia;
/// # fn main() {
/// let uri = cloudmedia::protocol::sip::Uri::sip(domain!("example.com"));
/// let domain = named_header!(uri);
/// # }
/// ```
#[macro_export]
macro_rules! named_header {
    ($u:tt) => {
        cloudmedia::protocol::sip::NamedHeader {
            display_name: None,
            uri: $u,
            params: ::std::collections::HashMap::new(),
        }
    };
    ($u:tt, $name:tt) => {
        cloudmedia::protocol::sip::NamedHeader {
            display_name: Some($name.into()),
            uri: $u,
            params: ::std::collections::HashMap::new()
        }
    };
}

/// Same as `named_header` except for internal use.
macro_rules! __named_header {
    ($u:tt) => {
        crate::proto::sip::NamedHeader {
            display_name: None,
            uri: $u,
            params: ::std::collections::HashMap::new(),
        }
    };
}
