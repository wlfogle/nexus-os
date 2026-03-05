#ifndef KERNEL_UDP_H
#define KERNEL_UDP_H

#include "netdev.h"

#define UDP_MAX_SOCKETS 16
#define UDP_MAX_PAYLOAD (MTU - sizeof(struct ipv4_header) - sizeof(struct udp_header))

/* UDP header (8 bytes) */
struct udp_header {
    uint16_t src_port;         /* Source port */
    uint16_t dest_port;        /* Destination port */
    uint16_t length;           /* Header + data length */
    uint16_t checksum;         /* Optional checksum */
} __attribute__((packed));

/* UDP socket state */
struct udp_socket {
    uint32_t id;               /* Socket ID */
    uint8_t in_use;
    uint16_t local_port;
    ipv4_addr_t local_ip;
    ipv4_addr_t remote_ip;
    uint16_t remote_port;
    
    /* Receive queue (simplified - store one packet) */
    struct net_packet *rx_packet;
    uint32_t rx_len;
};

/* Send UDP datagram */
int udp_send(struct netdev *dev, const ipv4_addr_t *dest_ip, uint16_t dest_port,
             uint16_t src_port, const uint8_t *data, uint32_t len);

/* Handle incoming UDP packet */
int udp_receive(struct netdev *dev, const ipv4_addr_t *src_ip, const uint8_t *data, uint32_t len);

/* UDP socket operations */
int udp_socket_create(uint16_t local_port);
int udp_socket_close(int socket_id);
int udp_socket_send(int socket_id, const ipv4_addr_t *dest_ip, uint16_t dest_port,
                    const uint8_t *data, uint32_t len);
int udp_socket_recv(int socket_id, uint8_t *buffer, uint32_t buf_len);

/* Initialize UDP module */
void udp_init(void);

#endif /* KERNEL_UDP_H */
