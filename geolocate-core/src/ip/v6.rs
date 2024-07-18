use std::net::Ipv6Addr;

/// An IPv6 address block.
pub type Ipv6AddrBlock = super::IpAddrBlock<Ipv6Addr>;

impl super::SealedIpAddr for Ipv6Addr {}
