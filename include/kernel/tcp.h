#ifndef KERNEL_TCP_H
#define KERNEL_TCP_H

#include "netdev.h"

#define TCP_MAX_SOCKETS 32
#define TCP_MAX_PAYLOAD (MTU - sizeof(struct ipv4_header) - sizeof(struct tcp_header))
#define TCP_WINDOW_SIZE 16384
#define TCP_MSS 1460
#define TCP_RETRANSMIT_TIMEOUT 1000  /* 1 second in ms */

/* TCP header (20 bytes minimum) */
struct tcp_header {
    uint16_t src_port;
    uint16_t dest_port;
    uint32_t seq_num;
    uint32_t ack_num;
    uint8_t data_offset;       /* 4 bits offset, 4 bits reserved */
    uint8_t flags;             /* FIN, SYN, RST, PSH, ACK, URG */
    uint16_t window_size;
    uint16_t checksum;
    uint16_t urgent_ptr;
} __attribute__((packed));

/* TCP flags */
#define TCP_FIN 0x01
#define TCP_SYN 0x02
#define TCP_RST 0x04
#define TCP_PSH 0x08
#define TCP_ACK 0x10
#define TCP_URG 0x20

/* TCP connection states */
typedef enum {
    TCP_CLOSED = 0,
    TCP_LISTEN = 1,
    TCP_SYN_SENT = 2,
    TCP_SYN_RECV = 3,
    TCP_ESTABLISHED = 4,
    TCP_FIN_WAIT_1 = 5,
    TCP_FIN_WAIT_2 = 6,
    TCP_CLOSE_WAIT = 7,
    TCP_CLOSING = 8,
    TCP_TIME_WAIT = 9
} tcp_state_t;

/* TCP socket structure */
struct tcp_socket {
    uint32_t id;
    uint8_t in_use;
    tcp_state_t state;
    
    uint16_t local_port;
    uint16_t remote_port;
    ipv4_addr_t local_ip;
    ipv4_addr_t remote_ip;
    
    /* Sequence numbers */
    uint32_t seq_num;          /* Next sequence to send */
    uint32_t ack_num;          /* Next sequence expected */
    uint32_t remote_seq;       /* Remote sequence number */
    
    /* Flow control */
    uint16_t window_size;
    uint32_t unacked_data;
    
    /* Receive buffer */
    uint8_t *rx_buffer;
    uint32_t rx_len;
    uint32_t rx_capacity;
    
    /* Retransmission */
    uint32_t retransmit_time;
    uint8_t retransmit_count;
};

/* Send TCP segment */
int tcp_send(struct netdev *dev, const ipv4_addr_t *dest_ip, uint16_t dest_port,
             uint16_t src_port, uint32_t seq_num, uint32_t ack_num,
             uint8_t flags, const uint8_t *data, uint32_t len);

/* Handle incoming TCP packet */
int tcp_receive(struct netdev *dev, const ipv4_addr_t *src_ip, const uint8_t *data, uint32_t len);

/* TCP socket operations */
int tcp_socket_create(void);
int tcp_socket_listen(int socket_id, uint16_t port);
int tcp_socket_connect(int socket_id, const ipv4_addr_t *dest_ip, uint16_t dest_port);
int tcp_socket_accept(int socket_id);
int tcp_socket_close(int socket_id);
int tcp_socket_send(int socket_id, const uint8_t *data, uint32_t len);
int tcp_socket_recv(int socket_id, uint8_t *buffer, uint32_t buf_len);

#endif /* KERNEL_TCP_H */
