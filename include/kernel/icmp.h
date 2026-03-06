#ifndef KERNEL_ICMP_H
#define KERNEL_ICMP_H

#include "netdev.h"

/* ICMP message types */
#define ICMP_ECHO_REPLY      0
#define ICMP_DEST_UNREACHABLE 3
#define ICMP_ECHO_REQUEST    8
#define ICMP_TTL_EXCEEDED   11

/* ICMP header (8 bytes minimum) */
struct icmp_header {
    uint8_t type;              /* Message type */
    uint8_t code;              /* Message subtype */
    uint16_t checksum;         /* Checksum */
    uint16_t identifier;       /* Identifier for echo */
    uint16_t sequence;         /* Sequence number for echo */
} __attribute__((packed));

/* ICMP packet with data */
struct icmp_echo_packet {
    struct icmp_header hdr;
    uint8_t data[1472];        /* Payload (1500 - 28 IPv4 - 8 ICMP) */
};

/* Send ICMP echo request (ping) */
int icmp_send_echo(struct netdev *dev, const ipv4_addr_t *dest_ip,
                   uint16_t id, uint16_t seq,
                   const uint8_t *data, uint32_t len);

/* Handle incoming ICMP packet */
int icmp_receive(struct netdev *dev, const ipv4_addr_t *src_ip, const uint8_t *data, uint32_t len);

#endif /* KERNEL_ICMP_H */
