#ifndef KERNEL_IPv4_H
#define KERNEL_IPv4_H

#include "netdev.h"

#define IPv4_PROTO_ICMP 1
#define IPv4_PROTO_TCP  6
#define IPv4_PROTO_UDP  17

/* IPv4 header (20 bytes minimum) */
struct ipv4_header {
    uint8_t version_ihl;        /* Version (4 bits) + IHL (4 bits) */
    uint8_t dscp_ecn;           /* DSCP (6 bits) + ECN (2 bits) */
    uint16_t total_length;      /* Total length (header + payload) */
    uint16_t identification;    /* For fragmentation */
    uint16_t flags_offset;      /* Flags (3 bits) + Fragment offset (13 bits) */
    uint8_t ttl;                /* Time To Live */
    uint8_t protocol;           /* Next protocol (ICMP, TCP, UDP, etc) */
    uint16_t checksum;          /* Checksum of header */
    ipv4_addr_t src_ip;
    ipv4_addr_t dest_ip;
} __attribute__((packed));

/* IPv4 address utility functions */
int ipv4_addr_equal(const ipv4_addr_t *a, const ipv4_addr_t *b);
void ipv4_addr_copy(ipv4_addr_t *dest, const ipv4_addr_t *src);
int ipv4_in_subnet(const ipv4_addr_t *ip, const ipv4_addr_t *network, const ipv4_addr_t *netmask);

/* IPv4 receive handler */
int ipv4_receive(struct netdev *dev, const uint8_t *data, uint32_t len);

/* IPv4 send */
int ipv4_send(struct netdev *dev, const ipv4_addr_t *dest_ip, uint8_t protocol,
              const uint8_t *payload, uint32_t len);

#endif /* KERNEL_IPv4_H */
