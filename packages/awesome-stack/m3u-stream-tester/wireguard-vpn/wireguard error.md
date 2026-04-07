[#] ip link add dev wg0-client type wireguard
[#] wg setconf wg0-client /dev/fd/63
[#] ip -4 address add 10.0.0.2/24 dev wg0-client
[#] ip link set mtu 1420 up dev wg0-client
[#] resolvconf -a wg0-client -m 0 -x
[#] wg set wg0-client fwmark 51820
[#] ip -4 rule add not fwmark 51820 table 51820
[#] ip -4 rule add table main suppress_prefixlength 0
[#] ip -4 route add 0.0.0.0/0 dev wg0-client table 51820
[#] sysctl -q net.ipv4.conf.all.src_valid_mark=1
[#] nft -f /dev/fd/63
