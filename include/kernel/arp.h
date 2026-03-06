#ifndef KERNEL_ARP_H
#define KERNEL_ARP_H

#include "netdev.h"

#define ARP_TABLE_SIZE 32

/* ARP opcodes */
#define ARP_OP_REQUEST 1
#define ARP_OP_REPLY   2

/* ARP packet */
struct arp_packet {
    uint16_t hw_type;        /* Hardware type (1 = Ethernet) */
    uint16_t proto_type;     /* Protocol type (0x0800 = IPv4) */
    uint8_t hw_addr_len;     /* Hardware address length (6) */
    uint8_t proto_addr_len;  /* Protocol address length (4) */
    uint16_t opcode;         /* Request or Reply */
    mac_addr_t sender_hw;
    ipv4_addr_t sender_proto;
    mac_addr_t target_hw;
    ipv4_addr_t target_proto;
} __attribute__((packed));

/* ARP cache entry */
struct arp_entry {
    ipv4_addr_t ip;
    mac_addr_t mac;
    uint32_t age;  /* Timestamp for expiry */
};

/* ARP functions */
void arp_init(void);
int arp_receive(struct netdev *dev, const uint8_t *data, uint32_t len);
int arp_request(struct netdev *dev, const ipv4_addr_t *target_ip);
int arp_resolve(struct netdev *dev, const ipv4_addr_t *ip, mac_addr_t *mac);

/* IPv4 address utilities (implemented in arp.c) */
int ipv4_addr_equal(const ipv4_addr_t *a, const ipv4_addr_t *b);
void ipv4_addr_copy(ipv4_addr_t *dest, const ipv4_addr_t *src);
int ipv4_in_subnet(const ipv4_addr_t *ip, const ipv4_addr_t *network, const ipv4_addr_t *netmask);

#endif /* KERNEL_ARP_H */
