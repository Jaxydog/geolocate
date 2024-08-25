use std::net::Ipv4Addr;

/// An IPv4 address block.
pub type Ipv4AddrBlock = super::IpAddrBlock<Ipv4Addr>;
/// An IPv4 address block map.
pub type Ipv4AddrBlockMap<T> = super::IpAddrBlockMap<Ipv4Addr, T>;

impl super::Address for Ipv4Addr {}

impl super::private::Sealed for Ipv4Addr {}
