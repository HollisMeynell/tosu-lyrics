use crate::model::http::network::NetAddress;
use std::net::{IpAddr, Ipv4Addr};

// 枚举接口拿不到网卡类型，虚拟/隧道网卡按名称粗判；漏判时由前端选择器兜底。
fn is_virtual_nic(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    const PREFIXES: &[&str] = &[
        "docker",
        "br-",
        "veth",
        "virbr",
        "vmnet",
        "vboxnet",
        "tun",
        "tap",
        "wg",
        "tailscale",
        "zt",
        "zerotier",
    ];
    const SNIPPETS: &[&str] = &[
        "loopback",
        "vethernet",
        "hyper-v",
        "hyper v",
        "wsl",
        "vmware",
        "virtualbox",
    ];
    lower == "lo"
        || PREFIXES.iter().any(|p| lower.starts_with(p))
        || SNIPPETS.iter().any(|s| lower.contains(s))
}

fn is_private_v4(v4: Ipv4Addr) -> bool {
    let o = v4.octets();
    o[0] == 10 || (o[0] == 172 && (16..=31).contains(&o[1])) || (o[0] == 192 && o[1] == 168)
}

fn is_link_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 169 && o[1] == 254
        }
        IpAddr::V6(v6) => (v6.segments()[0] & 0xffc0) == 0xfe80,
    }
}

/// 排序权重：真实网卡私有 IPv4 优先，链路本地垫底。
fn rank(ip: IpAddr, is_virtual: bool) -> u8 {
    if is_link_local(ip) {
        return 4;
    }
    match ip {
        IpAddr::V4(v4) if is_private_v4(v4) => {
            if is_virtual {
                1
            } else {
                0
            }
        }
        IpAddr::V4(_) => 2,
        IpAddr::V6(_) => 3,
    }
}

pub fn list_net_addresses() -> Vec<NetAddress> {
    let Ok(ifas) = local_ip_address::list_afinet_netifas() else {
        return Vec::new();
    };
    let mut items: Vec<(u8, String, String)> = Vec::new();
    for (name, ip) in ifas {
        if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
            continue;
        }
        let is_virtual = is_virtual_nic(&name);
        let item = (rank(ip, is_virtual), name, ip.to_string());
        match items.iter_mut().find(|(_, _, addr)| *addr == item.2) {
            Some(exist) => {
                if item.0 < exist.0 {
                    *exist = item;
                }
            }
            None => items.push(item),
        }
    }
    items.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.2.cmp(&b.2)));
    items
        .into_iter()
        .enumerate()
        .map(|(index, (_, name, address))| NetAddress {
            address,
            name,
            preferred: index == 0,
        })
        .collect()
}
