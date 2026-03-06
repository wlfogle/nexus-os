#ifndef KERNEL_ETHERNET_H
#define KERNEL_ETHERNET_H

#include "../libc/stdint.h"
#include "netdev.h"

#define ETH_TYPE_IPv4  0x0800
#define ETH_TYPE_ARP   0x0806

/* Ethernet frame header (14 bytes) */
struct eth_header {
    uint8_t dest_mac[6];
    uint8_t src_mac[6];
    uint16_t type;  /* Big-endian: IPv4, ARP, etc */
} __attribute__((packed));

/* Ethernet frame (header + payload + FCS) */
struct eth_frame {
    struct eth_header hdr;
    uint8_t payload[MTU];
    uint16_t fcs;  /* Frame check sequence (not validated in Phase 6) */
};

/* Send Ethernet frame */
int eth_send(struct netdev *dev, const mac_addr_t *dest_mac, uint16_t type,
             const uint8_t *payload, uint32_t len);

/* Handle incoming Ethernet frame */
int eth_receive(struct netdev *dev, struct net_packet *pkt);

/* Utility functions */
int eth_mac_equal(const mac_addr_t *a, const mac_addr_t *b);
void eth_mac_copy(mac_addr_t *dest, const mac_addr_t *src);
int eth_is_broadcast(const mac_addr_t *mac);
int eth_is_multicast(const mac_addr_t *mac);

/* Network byte order (big-endian) conversion */
static inline uint16_t htons(uint16_t x)
{
    return ((x & 0xFF) << 8) | ((x >> 8) & 0xFF);
}

static inline uint16_t ntohs(uint16_t x)
{
    return htons(x);  /* Same operation */
}

static inline uint32_t htonl(uint32_t x)
{
    return ((x & 0xFF) << 24) | ((x & 0xFF00) << 8) | ((x >> 8) & 0xFF00) | ((x >> 24) & 0xFF);
}

static inline uint32_t ntohl(uint32_t x)
{
    return htonl(x);  /* Same operation */
}

#endif /* KERNEL_ETHERNET_H */
