use std::net::Ipv4Addr;

/// An IPv4 address block.
pub type Ipv4AddrBlock = super::IpAddrBlock<Ipv4Addr>;

impl super::SealedIpAddr for Ipv4Addr {}
