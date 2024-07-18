use std::net::Ipv6Addr;

/// An IPv6 address block.
pub type Ipv6AddrBlock = super::IpAddrBlock<Ipv6Addr>;
/// An IPv6 address block map.
pub type Ipv6AddrBlockMap<T> = super::IpAddrBlockMap<Ipv6Addr, T>;

impl super::SealedIpAddr for Ipv6Addr {}
