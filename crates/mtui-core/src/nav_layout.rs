//! Navigation order and labels inside each top-level group.
//!
//! Resources register in the order their feature was implemented, which
//! scatters related menus (IP shows DHCP, Firewall, DNS and `IPsec` items
//! interleaved). This table lists each group's items the way `WinBox`
//! arranges its sub-menus: sections in alphabetical order, the section's main
//! screen first, and its tabs right after it. Items that belong to a section
//! carry a `Section › Tab` label so siblings such as IP Peers or Leases say
//! which menu they come from.
//!
//! Resources missing from a group's layout keep their catalog order and label
//! and are appended after the listed ones, so a new screen still shows up
//! before it is placed here.

/// One nav row: resource id and the label shown for it.
pub type NavEntry = (&'static str, &'static str);

/// Per-group layout, keyed by `NavGroup::id`.
pub static NAV_LAYOUT: &[(&str, &[NavEntry])] = &[
    ("interfaces-group", INTERFACES),
    ("ppp-group", PPP),
    ("ip-group", IP),
    ("ipv6-group", IPV6),
    ("routing-group", ROUTING),
    ("tools-group", TOOLS),
    ("system-group", SYSTEM),
];

/// Layout for `group`, when one is defined.
#[must_use]
pub fn group_layout(group: &str) -> Option<&'static [NavEntry]> {
    NAV_LAYOUT
        .iter()
        .find(|(id, _)| *id == group)
        .map(|(_, entries)| *entries)
}

const INTERFACES: &[NavEntry] = &[
    ("interfaces", "Interface"),
    ("ethernet", "Ethernet"),
    ("bonding", "Bonding"),
    ("vlan", "VLAN"),
    ("vxlan", "VXLAN"),
    ("vrrp", "VRRP"),
    ("macvlan", "MACVLAN"),
    ("veth", "VETH"),
    ("vrf", "VRF"),
    ("eoip", "EoIP Tunnel"),
    ("gre", "GRE Tunnel"),
    ("gre6", "GRE6 Tunnel"),
    ("ipip", "IP Tunnel"),
    ("6to4", "6to4 Tunnel"),
    ("interface-lists", "Interface List"),
    ("interface-list-members", "Interface List › Members"),
    ("lte", "LTE"),
    ("lte-apn", "LTE › APN"),
    ("macsec", "MACsec"),
    ("macsec-profiles", "MACsec › Profiles"),
    ("wifi", "WiFi"),
    ("wifi-configuration", "WiFi › Configuration"),
    ("wifi-security", "WiFi › Security"),
    ("wifi-channel", "WiFi › Channel"),
    ("wifi-datapath", "WiFi › Datapath"),
    ("wifi-provisioning", "WiFi › Provisioning"),
    ("wifi-registration-table", "WiFi › Registration"),
    ("wifi-cap", "WiFi › CAP"),
    ("wifi-capsman", "WiFi › CAPsMAN"),
    ("wireless", "Wireless"),
    ("wireless-security-profiles", "Wireless › Sec. Profiles"),
    ("wireless-access-list", "Wireless › Access List"),
    ("wireless-registration-table", "Wireless › Registration"),
    ("detect-internet", "Detect Internet"),
];

const PPP: &[NavEntry] = &[
    ("ppp-secrets", "Secrets"),
    ("ppp-profiles", "Profiles"),
    ("ppp-active", "Active Connections"),
    ("ppp-aaa", "AAA"),
    ("ppp-client", "PPP Client"),
    ("pppoe-servers", "PPPoE Servers"),
    ("pppoe-server-ifaces", "PPPoE Sessions"),
    ("pppoe-clients", "PPPoE Clients"),
    ("l2tp-server", "L2TP Server"),
    ("l2tp-server-ifaces", "L2TP Sessions"),
    ("l2tp-client", "L2TP Clients"),
    ("ovpn-server", "OpenVPN Server"),
    ("ovpn-server-ifaces", "OpenVPN Sessions"),
    ("ovpn-client", "OpenVPN Clients"),
    ("pptp-server", "PPTP Server"),
    ("pptp-server-ifaces", "PPTP Sessions"),
    ("pptp-client", "PPTP Clients"),
    ("sstp-server", "SSTP Server"),
    ("sstp-server-ifaces", "SSTP Sessions"),
    ("sstp-client", "SSTP Clients"),
];

const IP: &[NavEntry] = &[
    ("addresses", "Addresses"),
    ("arp", "ARP"),
    ("cloud", "Cloud"),
    ("dhcp-clients", "DHCP Client"),
    ("dhcp-relay", "DHCP Relay"),
    ("dhcp-servers", "DHCP Server"),
    ("dhcp-networks", "DHCP Server › Networks"),
    ("dhcp-leases", "DHCP Server › Leases"),
    ("dhcp-options", "DHCP Server › Options"),
    ("dhcp-option-sets", "DHCP Server › Opt. Sets"),
    ("dhcp-alerts", "DHCP Server › Alerts"),
    ("dns", "DNS"),
    ("dns-static", "DNS › Static"),
    ("dns-cache", "DNS › Cache"),
    ("firewall-filter", "Firewall › Filter Rules"),
    ("firewall-nat", "Firewall › NAT"),
    ("firewall-mangle", "Firewall › Mangle"),
    ("firewall-raw", "Firewall › Raw"),
    ("firewall-connections", "Firewall › Connections"),
    ("connection-tracking", "Firewall › Conntrack"),
    ("address-list", "Firewall › Address Lists"),
    ("firewall-layer7", "Firewall › Layer7"),
    ("firewall-service-port", "Firewall › Service Ports"),
    ("hotspot", "Hotspot"),
    ("hotspot-profiles", "Hotspot › Profiles"),
    ("hotspot-users", "Hotspot › Users"),
    ("hotspot-user-profiles", "Hotspot › User Profiles"),
    ("hotspot-hosts", "Hotspot › Hosts"),
    ("hotspot-ip-bindings", "Hotspot › IP Bindings"),
    ("hotspot-cookies", "Hotspot › Cookies"),
    ("hotspot-walled-garden", "Hotspot › Walled Garden"),
    ("hotspot-walled-garden-ip", "Hotspot › W. Garden IP"),
    ("igmp-proxy", "IGMP Proxy"),
    ("igmp-proxy-interfaces", "IGMP Proxy › Interfaces"),
    ("igmp-proxy-mfc", "IGMP Proxy › MFC"),
    ("ipsec-policies", "IPsec › Policies"),
    ("ipsec-peers", "IPsec › Peers"),
    ("ipsec-identities", "IPsec › Identities"),
    ("ipsec-profiles", "IPsec › Profiles"),
    ("ipsec-proposals", "IPsec › Proposals"),
    ("ipsec-installed-sa", "IPsec › Installed SAs"),
    ("ipsec-mode-config", "IPsec › Mode Configs"),
    ("ipsec-key-psk", "IPsec › Keys PSK"),
    ("ipsec-key-rsa", "IPsec › Keys RSA"),
    ("ipsec-key-qkd", "IPsec › Keys QKD"),
    ("ipsec-settings", "IPsec › Settings"),
    ("kid-control", "Kid Control"),
    ("kid-control-devices", "Kid Control › Devices"),
    ("neighbors", "Neighbors"),
    ("neighbor-discovery", "Neighbors › Discovery"),
    ("pools", "Pool"),
    ("proxy", "Proxy"),
    ("proxy-access", "Proxy › Access"),
    ("proxy-cache", "Proxy › Cache"),
    ("proxy-direct", "Proxy › Direct"),
    ("routes", "Routes"),
    ("ip-services", "Services"),
    ("ip-settings", "Settings"),
    ("smb", "SMB"),
    ("smb-shares", "SMB › Shares"),
    ("smb-users", "SMB › Users"),
    ("socks", "SOCKS"),
    ("ip-ssh", "SSH"),
    ("traffic-flow", "Traffic Flow"),
    ("traffic-flow-targets", "Traffic Flow › Targets"),
    ("traffic-flow-ipfix", "Traffic Flow › IPFIX"),
    ("upnp", "UPnP"),
    ("upnp-interfaces", "UPnP › Interfaces"),
];

const IPV6: &[NavEntry] = &[
    ("ipv6-addresses", "Addresses"),
    ("ipv6-dhcp-client", "DHCP Client"),
    ("ipv6-dhcp-relay", "DHCP Relay"),
    ("ipv6-dhcp-server", "DHCP Server"),
    ("ipv6-dhcp-bindings", "DHCP Server › Bindings"),
    ("ipv6-firewall-filter", "Firewall › Filter Rules"),
    ("ipv6-firewall-nat", "Firewall › NAT"),
    ("ipv6-firewall-mangle", "Firewall › Mangle"),
    ("ipv6-firewall-raw", "Firewall › Raw"),
    ("ipv6-firewall-connections", "Firewall › Connections"),
    ("ipv6-address-list", "Firewall › Address Lists"),
    ("ipv6-nd", "ND"),
    ("ipv6-nd-prefix", "ND › Prefixes"),
    ("ipv6-neighbors", "Neighbors"),
    ("ipv6-pool", "Pool"),
    ("ipv6-routes", "Routes"),
    ("ipv6-settings", "Settings"),
];

const ROUTING: &[NavEntry] = &[
    ("bfd", "BFD"),
    ("bgp-connections", "BGP › Connections"),
    ("bgp-sessions", "BGP › Sessions"),
    ("bgp-templates", "BGP › Templates"),
    ("bgp-advertisements", "BGP › Advertisements"),
    ("routing-filters", "Filters"),
    ("ospf-instances", "OSPF › Instances"),
    ("ospf-areas", "OSPF › Areas"),
    ("ospf-interface-templates", "OSPF › Templates"),
    ("ospf-interfaces", "OSPF › Interfaces"),
    ("ospf-neighbors", "OSPF › Neighbors"),
    ("ospf-lsa", "OSPF › LSA"),
    ("rip-instances", "RIP › Instances"),
    ("rip-interface-templates", "RIP › Templates"),
    ("routing-id", "Router ID"),
    ("routing-rules", "Rules"),
    ("routing-tables", "Tables"),
];

const TOOLS: &[NavEntry] = &[
    ("bandwidth-test", "Bandwidth Test"),
    ("email", "Email"),
    ("flood-ping", "Flood Ping"),
    ("graphing", "Graphing"),
    ("graphing-interface", "Graphing › Interfaces"),
    ("graphing-queue", "Graphing › Queues"),
    ("graphing-resource", "Graphing › Resources"),
    ("ip-scan", "IP Scan"),
    ("mac-scan", "MAC Scan"),
    ("netwatch", "Netwatch"),
    ("sniffer", "Packet Sniffer"),
    ("ping", "Ping"),
    ("profiler", "Profiler"),
    ("romon", "RoMON"),
    ("romon-ports", "RoMON › Ports"),
    ("sms", "SMS"),
    ("traceroute", "Traceroute"),
    ("wol", "Wake on LAN"),
];

const SYSTEM: &[NavEntry] = &[
    ("certificates", "Certificates"),
    ("clock", "Clock"),
    ("system-console", "Console"),
    ("device-mode", "Device Mode"),
    ("disks", "Disks"),
    ("health", "Health"),
    ("history", "History"),
    ("identity", "Identity"),
    ("leds", "LEDs"),
    ("led-settings", "LEDs › Settings"),
    ("license", "License"),
    ("logging", "Logging"),
    ("logging-actions", "Logging › Actions"),
    ("logs", "Logs"),
    ("note", "Note"),
    ("ntp", "NTP Client"),
    ("ntp-server", "NTP Server"),
    ("ntp-keys", "NTP Keys"),
    ("packages", "Packages"),
    ("package-update", "Packages › Update"),
    ("ports", "Ports"),
    ("resources", "Resources"),
    ("routerboard", "RouterBOARD"),
    ("routerboard-settings", "RouterBOARD › Settings"),
    ("routerboard-mode-button", "RouterBOARD › Mode Btn"),
    ("routerboard-reset-button", "RouterBOARD › Reset Btn"),
    ("scheduler", "Scheduler"),
    ("scripts", "Scripts"),
    ("snmp", "SNMP"),
    ("snmp-communities", "SNMP › Communities"),
    ("special-login", "Special Login"),
    ("users", "Users"),
    ("user-groups", "Users › Groups"),
    ("ssh-keys", "Users › SSH Keys"),
    ("watchdog", "Watchdog"),
    ("reset-configuration", "Reset Configuration"),
    ("reboot", "Reboot"),
    ("shutdown", "Shutdown"),
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::resources::{ALL_RESOURCES, NAVIGATION, navigation_tree};

    /// Longest label that fits a child row in the medium layout (28-column
    /// nav pane minus border and child indent).
    const NAV_LABEL_MAX_WIDTH: usize = 24;

    #[test]
    fn layout_ids_exist_in_their_group_once() {
        for (group, entries) in NAV_LAYOUT {
            assert!(
                NAVIGATION.iter().any(|nav| nav.id == *group),
                "unknown group {group}"
            );
            let mut seen = HashSet::new();
            for (id, _) in *entries {
                assert!(seen.insert(*id), "{id} listed twice in {group}");
                let spec = ALL_RESOURCES
                    .iter()
                    .find(|spec| spec.id == *id)
                    .unwrap_or_else(|| panic!("{id} is not a resource"));
                assert_eq!(spec.group, *group, "{id} belongs to {}", spec.group);
            }
        }
    }

    #[test]
    fn layout_covers_every_resource_of_its_group() {
        for (group, entries) in NAV_LAYOUT {
            for spec in ALL_RESOURCES.iter().filter(|spec| spec.group == *group) {
                assert!(
                    entries.iter().any(|(id, _)| *id == spec.id),
                    "{} missing from the {group} layout",
                    spec.id
                );
            }
        }
    }

    #[test]
    fn layout_labels_fit_the_medium_nav_pane() {
        for (_, entries) in NAV_LAYOUT {
            for (id, label) in *entries {
                assert!(
                    label.chars().count() <= NAV_LABEL_MAX_WIDTH,
                    "{id}: {label:?} is wider than {NAV_LABEL_MAX_WIDTH}"
                );
            }
        }
    }

    #[test]
    fn ip_group_keeps_sections_together() {
        let tree = navigation_tree();
        let ip = tree
            .iter()
            .find(|item| item.id == "ip-group")
            .expect("ip group");
        let labels: Vec<&str> = ip.children.iter().map(|c| c.label.as_str()).collect();
        let section_of = |label: &str| label.split(" › ").next().unwrap_or(label).to_string();
        let mut closed = HashSet::new();
        let mut current = String::new();
        for label in &labels {
            let section = section_of(label);
            if section != current {
                assert!(
                    closed.insert(current.clone()) || current.is_empty(),
                    "section {current} split: {labels:?}"
                );
                assert!(
                    !closed.contains(&section),
                    "section {section} split: {labels:?}"
                );
                current = section;
            }
        }
        let leases = labels.iter().position(|l| *l == "DHCP Server › Leases");
        let server = labels.iter().position(|l| *l == "DHCP Server");
        assert!(server < leases, "{labels:?}");
    }
}
