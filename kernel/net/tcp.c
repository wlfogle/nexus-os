#include "../../include/kernel/tcp.h"
#include "../../include/kernel/ipv4.h"
#include "../../include/kernel/ethernet.h"
#include "../../include/kernel/netdev.h"
#include <string.h>

/* Kernel memory allocation */
extern void *kmalloc(uint32_t size);
extern void kfree(void *ptr);

/* Global TCP socket table */
static struct tcp_socket tcp_sockets[TCP_MAX_SOCKETS];
static uint16_t tcp_next_ephemeral_port = 49152;

/* Random ISN (Initial Sequence Number) - in production use proper crypto */
static uint32_t tcp_initial_seq = 1000000;

/* Initialize TCP layer */
static void tcp_init(void) {
    static uint8_t initialized = 0;
    if (initialized) return;
    
    for (int i = 0; i < TCP_MAX_SOCKETS; i++) {
        tcp_sockets[i].id = i;
        tcp_sockets[i].in_use = 0;
        tcp_sockets[i].state = TCP_CLOSED;
        tcp_sockets[i].rx_capacity = 65536;
        tcp_sockets[i].rx_buffer = kmalloc(65536);
        tcp_sockets[i].rx_len = 0;
        tcp_sockets[i].retransmit_count = 0;
    }
    initialized = 1;
}

/* Compute TCP checksum (pseudo-header + TCP segment) */
static uint16_t tcp_checksum(const ipv4_addr_t *src_ip, const ipv4_addr_t *dest_ip,
                             const struct tcp_header *tcp_hdr, const uint8_t *data, uint32_t len) {
    uint32_t sum = 0;
    
    /* Pseudo-header: source IP */
    sum += (src_ip->addr[0] << 8) | src_ip->addr[1];
    sum += (src_ip->addr[2] << 8) | src_ip->addr[3];
    /* Destination IP */
    sum += (dest_ip->addr[0] << 8) | dest_ip->addr[1];
    sum += (dest_ip->addr[2] << 8) | dest_ip->addr[3];
    sum += 0x0006;  /* TCP protocol number */
    sum += htons(sizeof(struct tcp_header) + len);
    
    /* TCP header */
    const uint16_t *ptr = (const uint16_t *)tcp_hdr;
    for (uint32_t i = 0; i < 10; i++) {  /* 20 bytes / 2 */
        if (i != 8) {  /* Skip checksum field */
            sum += ptr[i];
        }
    }
    
    /* Data */
    ptr = (const uint16_t *)data;
    for (uint32_t i = 0; i < len / 2; i++) {
        sum += ptr[i];
    }
    if (len % 2) {
        sum += ((const uint8_t *)data)[len - 1] << 8;
    }
    
    /* Fold 32-bit sum to 16-bit */
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    return ~sum;
}

/* Send TCP segment */
int tcp_send(struct netdev *dev, const ipv4_addr_t *dest_ip, uint16_t dest_port,
             uint16_t src_port, uint32_t seq_num, uint32_t ack_num,
             uint8_t flags, const uint8_t *data, uint32_t len) {
    if (!dev || !dest_ip) return -1;
    if (len > TCP_MAX_PAYLOAD) return -1;
    
    struct net_packet *pkt = netdev_alloc_packet();
    if (!pkt) return -1;
    
    /* Build TCP header */
    struct tcp_header *tcp_hdr = (struct tcp_header *)pkt->data;
    
    tcp_hdr->src_port = htons(src_port);
    tcp_hdr->dest_port = htons(dest_port);
    tcp_hdr->seq_num = htonl(seq_num);
    tcp_hdr->ack_num = htonl(ack_num);
    tcp_hdr->data_offset = (sizeof(struct tcp_header) / 4) << 4;
    tcp_hdr->flags = flags;
    tcp_hdr->window_size = htons(TCP_WINDOW_SIZE);
    tcp_hdr->checksum = 0;
    tcp_hdr->urgent_ptr = 0;
    
    /* Copy payload */
    if (data && len > 0) {
        memcpy((uint8_t *)tcp_hdr + sizeof(struct tcp_header), data, len);
    }
    
    /* Compute checksum */
    tcp_hdr->checksum = tcp_checksum(&dev->ip_addr, dest_ip, tcp_hdr, data, len);
    
    /* Send through IPv4 */
    uint32_t tcp_len = sizeof(struct tcp_header) + len;
    int ret = ipv4_send(dev, dest_ip, IPv4_PROTO_TCP, pkt->data, tcp_len);
    netdev_free_packet(pkt);
    return ret;
}

/* Validate TCP checksum */
static int tcp_validate_checksum(const ipv4_addr_t *src_ip, const ipv4_addr_t *dest_ip,
                                 const struct tcp_header *tcp_hdr, const uint8_t *data, uint32_t len) {
    uint16_t received_checksum = tcp_hdr->checksum;
    uint16_t calculated = tcp_checksum(src_ip, dest_ip, tcp_hdr, data, len);
    return received_checksum == calculated;
}

/* Find socket by local port and remote address */
static struct tcp_socket *tcp_find_socket(uint16_t local_port, const ipv4_addr_t *remote_ip,
                                          uint16_t remote_port) {
    for (int i = 0; i < TCP_MAX_SOCKETS; i++) {
        if (!tcp_sockets[i].in_use) continue;
        if (tcp_sockets[i].local_port == local_port &&
            tcp_sockets[i].remote_ip.addr[0] == remote_ip->addr[0] &&
            tcp_sockets[i].remote_ip.addr[1] == remote_ip->addr[1] &&
            tcp_sockets[i].remote_ip.addr[2] == remote_ip->addr[2] &&
            tcp_sockets[i].remote_ip.addr[3] == remote_ip->addr[3] &&
            tcp_sockets[i].remote_port == remote_port) {
            return &tcp_sockets[i];
        }
    }
    return NULL;
}

/* Find socket in LISTEN state by port */
static struct tcp_socket *tcp_find_listen_socket(uint16_t port) {
    for (int i = 0; i < TCP_MAX_SOCKETS; i++) {
        if (!tcp_sockets[i].in_use) continue;
        if (tcp_sockets[i].local_port == port && tcp_sockets[i].state == TCP_LISTEN) {
            return &tcp_sockets[i];
        }
    }
    return NULL;
}

/* Handle SYN (connection request) */
static int tcp_handle_syn(struct netdev *dev, const ipv4_addr_t *src_ip,
                          const struct tcp_header *tcp_hdr, uint16_t dest_port) {
    struct tcp_socket *listen_sock = tcp_find_listen_socket(dest_port);
    if (!listen_sock) {
        /* No listening socket - send RST */
        tcp_send(dev, src_ip, ntohs(tcp_hdr->src_port), dest_port,
                 0, ntohs(tcp_hdr->seq_num) + 1, TCP_RST | TCP_ACK, NULL, 0);
        return -1;
    }
    
    /* Find or create connection socket */
    struct tcp_socket *sock = NULL;
    for (int i = 0; i < TCP_MAX_SOCKETS; i++) {
        if (!tcp_sockets[i].in_use) {
            sock = &tcp_sockets[i];
            break;
        }
    }
    
    if (!sock) {
        /* No available sockets */
        tcp_send(dev, src_ip, ntohs(tcp_hdr->src_port), dest_port,
                 0, ntohs(tcp_hdr->seq_num) + 1, TCP_RST | TCP_ACK, NULL, 0);
        return -1;
    }
    
    /* Initialize socket */
    sock->in_use = 1;
    sock->state = TCP_SYN_RECV;
    sock->local_port = dest_port;
    sock->remote_port = ntohs(tcp_hdr->src_port);
    sock->local_ip = dev->ip_addr;
    sock->remote_ip = *src_ip;
    sock->remote_seq = ntohs(tcp_hdr->seq_num);
    sock->ack_num = sock->remote_seq + 1;
    sock->seq_num = tcp_initial_seq++;
    sock->window_size = TCP_WINDOW_SIZE;
    sock->unacked_data = 0;
    sock->rx_len = 0;
    
    /* Send SYN-ACK */
    tcp_send(dev, src_ip, sock->remote_port, sock->local_port,
             sock->seq_num, sock->ack_num, TCP_SYN | TCP_ACK, NULL, 0);
    
    return 0;
}

/* Handle ACK for SYN-RECV */
static int tcp_handle_synack_ack(struct netdev *dev, const struct tcp_header *tcp_hdr, struct tcp_socket *sock) {
    uint32_t ack = ntohl(tcp_hdr->ack_num);
    
    if (ack != sock->seq_num + 1) {
        /* Invalid ACK number */
        return -1;
    }
    
    sock->state = TCP_ESTABLISHED;
    sock->seq_num++;
    return 0;
}

/* Handle data in ESTABLISHED state */
static int tcp_handle_established_data(struct netdev *dev, const ipv4_addr_t *src_ip,
                                       struct tcp_socket *sock,
                                       const struct tcp_header *tcp_hdr,
                                       const uint8_t *data, uint32_t len) {
    uint32_t seq = ntohl(tcp_hdr->seq_num);
    
    /* Check if this is the expected sequence number */
    if (seq != sock->ack_num) {
        /* Out of order - ignore for now (simplified TCP) */
        return -1;
    }
    
    /* Add data to receive buffer */
    if (len > 0) {
        if (sock->rx_len + len > sock->rx_capacity) {
            return -1;  /* Buffer full */
        }
        memcpy(sock->rx_buffer + sock->rx_len, data, len);
        sock->rx_len += len;
    }
    
    sock->ack_num += len;
    sock->remote_seq = seq;
    
    /* Send ACK */
    tcp_send(dev, src_ip, sock->remote_port, sock->local_port,
             sock->seq_num, sock->ack_num, TCP_ACK, NULL, 0);
    
    return 0;
}

/* Handle FIN (connection close) */
static int tcp_handle_fin(struct netdev *dev, const ipv4_addr_t *src_ip,
                          struct tcp_socket *sock, const struct tcp_header *tcp_hdr) {
    uint32_t seq = ntohl(tcp_hdr->seq_num);
    
    if (seq != sock->ack_num) {
        return -1;
    }
    
    sock->ack_num++;
    
    switch (sock->state) {
        case TCP_ESTABLISHED:
            sock->state = TCP_CLOSE_WAIT;
            /* Send ACK */
            tcp_send(dev, src_ip, sock->remote_port, sock->local_port,
                     sock->seq_num, sock->ack_num, TCP_ACK, NULL, 0);
            break;
            
        case TCP_FIN_WAIT_1:
            sock->state = TCP_TIME_WAIT;
            /* Send ACK */
            tcp_send(dev, src_ip, sock->remote_port, sock->local_port,
                     sock->seq_num, sock->ack_num, TCP_ACK, NULL, 0);
            break;
            
        case TCP_FIN_WAIT_2:
            sock->state = TCP_TIME_WAIT;
            /* Send ACK */
            tcp_send(dev, src_ip, sock->remote_port, sock->local_port,
                     sock->seq_num, sock->ack_num, TCP_ACK, NULL, 0);
            break;
            
        default:
            return -1;
    }
    
    return 0;
}

/* Handle incoming TCP packet */
int tcp_receive(struct netdev *dev, const ipv4_addr_t *src_ip, const uint8_t *data, uint32_t len) {
    tcp_init();
    
    if (len < sizeof(struct tcp_header)) {
        return -1;
    }
    
    const struct tcp_header *tcp_hdr = (const struct tcp_header *)data;
    uint16_t dest_port = ntohs(tcp_hdr->dest_port);
    uint16_t src_port = ntohs(tcp_hdr->src_port);
    uint8_t flags = tcp_hdr->flags;
    
    /* Calculate header length */
    uint8_t data_offset = tcp_hdr->data_offset >> 4;
    uint32_t hdr_len = data_offset * 4;
    if (hdr_len < 20 || hdr_len > len) {
        return -1;
    }
    
    const uint8_t *payload = data + hdr_len;
    uint32_t payload_len = len - hdr_len;
    
    /* Validate checksum */
    if (!tcp_validate_checksum(src_ip, &dev->ip_addr, tcp_hdr, payload, payload_len)) {
        return -1;
    }
    
    /* Find existing connection */
    struct tcp_socket *sock = tcp_find_socket(dest_port, src_ip, src_port);
    
    if (!sock) {
        /* No existing connection */
        if (flags & TCP_SYN) {
            return tcp_handle_syn(dev, src_ip, tcp_hdr, dest_port);
        }
        /* Send RST for non-SYN packets without connection */
        tcp_send(dev, src_ip, src_port, dest_port,
                 0, ntohs(tcp_hdr->seq_num) + 1, TCP_RST, NULL, 0);
        return -1;
    }
    
    /* Handle flags */
    if (flags & TCP_RST) {
        sock->state = TCP_CLOSED;
        sock->in_use = 0;
        return 0;
    }
    
    if (flags & TCP_SYN) {
        if (sock->state == TCP_SYN_RECV) {
            /* Retransmitted SYN, ignore */
            return 0;
        }
        /* Invalid SYN in established connection */
        tcp_send(dev, src_ip, src_port, dest_port,
                 ntohl(tcp_hdr->ack_num), ntohs(tcp_hdr->seq_num) + 1, TCP_RST, NULL, 0);
        return -1;
    }
    
    if (flags & TCP_ACK) {
        if (sock->state == TCP_SYN_RECV) {
            if (tcp_handle_synack_ack(dev, tcp_hdr, sock) < 0) {
                return -1;
            }
        }
    }
    
    if (payload_len > 0 && (sock->state == TCP_ESTABLISHED || sock->state == TCP_CLOSE_WAIT)) {
        tcp_handle_established_data(dev, src_ip, sock, tcp_hdr, payload, payload_len);
    }
    
    if (flags & TCP_FIN) {
        tcp_handle_fin(dev, src_ip, sock, tcp_hdr);
    }
    
    return 0;
}

/* Create TCP socket */
int tcp_socket_create(void) {
    tcp_init();
    
    for (int i = 0; i < TCP_MAX_SOCKETS; i++) {
        if (!tcp_sockets[i].in_use) {
            tcp_sockets[i].in_use = 1;
            tcp_sockets[i].state = TCP_CLOSED;
            tcp_sockets[i].rx_len = 0;
            tcp_sockets[i].seq_num = tcp_initial_seq++;
            tcp_sockets[i].unacked_data = 0;
            tcp_sockets[i].retransmit_count = 0;
            return i;
        }
    }
    return -1;
}

/* Listen on port */
int tcp_socket_listen(int socket_id, uint16_t port) {
    if (socket_id < 0 || socket_id >= TCP_MAX_SOCKETS) return -1;
    
    struct tcp_socket *sock = &tcp_sockets[socket_id];
    if (!sock->in_use || sock->state != TCP_CLOSED) return -1;
    
    /* Check if port already in use */
    for (int i = 0; i < TCP_MAX_SOCKETS; i++) {
        if (i != socket_id && tcp_sockets[i].in_use && tcp_sockets[i].local_port == port) {
            return -1;
        }
    }
    
    sock->state = TCP_LISTEN;
    sock->local_port = port;
    /* Will bind to first available device on connect/accept */
    
    return 0;
}

/* Connect to remote host */
int tcp_socket_connect(int socket_id, const ipv4_addr_t *dest_ip, uint16_t dest_port) {
    if (socket_id < 0 || socket_id >= TCP_MAX_SOCKETS) return -1;
    if (!dest_ip || dest_port == 0) return -1;
    
    struct tcp_socket *sock = &tcp_sockets[socket_id];
    if (!sock->in_use || sock->state != TCP_CLOSED) return -1;
    
    /* Need a device to send through - use first available */
    struct netdev *dev = NULL;
    for (int i = 0; i < 8; i++) {
        dev = netdev_get(i);
        if (dev && (dev->flags & IFF_UP)) break;
    }
    if (!dev) return -1;
    
    sock->state = TCP_SYN_SENT;
    sock->local_port = tcp_next_ephemeral_port++;
    sock->local_ip = dev->ip_addr;
    sock->remote_ip = *dest_ip;
    sock->remote_port = dest_port;
    sock->ack_num = 0;
    sock->seq_num = tcp_initial_seq++;
    sock->window_size = TCP_WINDOW_SIZE;
    
    /* Send SYN */
    return tcp_send(dev, dest_ip, dest_port, sock->local_port,
                    sock->seq_num, 0, TCP_SYN, NULL, 0);
}

/* Accept incoming connection (synchronous for simplicity) */
int tcp_socket_accept(int socket_id) {
    if (socket_id < 0 || socket_id >= TCP_MAX_SOCKETS) return -1;
    
    struct tcp_socket *listen_sock = &tcp_sockets[socket_id];
    if (!listen_sock->in_use || listen_sock->state != TCP_LISTEN) return -1;
    
    /* Find an established connection from this listening socket */
    for (int i = 0; i < TCP_MAX_SOCKETS; i++) {
        if (i != socket_id && tcp_sockets[i].in_use &&
            tcp_sockets[i].state == TCP_ESTABLISHED &&
            tcp_sockets[i].local_port == listen_sock->local_port &&
            tcp_sockets[i].local_ip.addr[0] == listen_sock->local_ip.addr[0] &&
            tcp_sockets[i].local_ip.addr[1] == listen_sock->local_ip.addr[1] &&
            tcp_sockets[i].local_ip.addr[2] == listen_sock->local_ip.addr[2] &&
            tcp_sockets[i].local_ip.addr[3] == listen_sock->local_ip.addr[3]) {
            return i;
        }
    }
    
    return -1;  /* No pending connections */
}

/* Close socket */
int tcp_socket_close(int socket_id) {
    if (socket_id < 0 || socket_id >= TCP_MAX_SOCKETS) return -1;
    
    struct tcp_socket *sock = &tcp_sockets[socket_id];
    if (!sock->in_use) return -1;
    
    if (sock->state == TCP_ESTABLISHED) {
        /* Need device for FIN */
        struct netdev *dev = netdev_get(0);
        if (dev) {
            sock->state = TCP_FIN_WAIT_1;
            tcp_send(dev, &sock->remote_ip, sock->remote_port, sock->local_port,
                     sock->seq_num, sock->ack_num, TCP_FIN | TCP_ACK, NULL, 0);
            sock->seq_num++;
            return 0;
        }
    }
    
    sock->state = TCP_CLOSED;
    sock->in_use = 0;
    sock->rx_len = 0;
    return 0;
}

/* Send data on socket */
int tcp_socket_send(int socket_id, const uint8_t *data, uint32_t len) {
    if (socket_id < 0 || socket_id >= TCP_MAX_SOCKETS) return -1;
    if (!data || len == 0) return 0;
    
    struct tcp_socket *sock = &tcp_sockets[socket_id];
    if (!sock->in_use || sock->state != TCP_ESTABLISHED) return -1;
    
    /* Need device */
    struct netdev *dev = netdev_get(0);
    if (!dev) return -1;
    
    uint32_t sent = 0;
    uint32_t remaining = len;
    
    while (remaining > 0) {
        uint32_t chunk = (remaining > TCP_MAX_PAYLOAD) ? TCP_MAX_PAYLOAD : remaining;
        
        int result = tcp_send(dev, &sock->remote_ip, sock->remote_port, sock->local_port,
                             sock->seq_num, sock->ack_num, TCP_PSH | TCP_ACK,
                             data + sent, chunk);
        if (result < 0) break;
        
        sock->seq_num += chunk;
        sock->unacked_data += chunk;
        sent += chunk;
        remaining -= chunk;
    }
    
    return sent;
}

/* Receive data from socket */
int tcp_socket_recv(int socket_id, uint8_t *buffer, uint32_t buf_len) {
    if (socket_id < 0 || socket_id >= TCP_MAX_SOCKETS) return -1;
    if (!buffer || buf_len == 0) return 0;
    
    struct tcp_socket *sock = &tcp_sockets[socket_id];
    if (!sock->in_use) return -1;
    
    uint32_t to_read = (sock->rx_len < buf_len) ? sock->rx_len : buf_len;
    
    if (to_read > 0) {
        memcpy(buffer, sock->rx_buffer, to_read);
        
        /* Shift remaining data */
        if (to_read < sock->rx_len) {
            uint32_t remaining = sock->rx_len - to_read;
            for (uint32_t i = 0; i < remaining; i++) {
                sock->rx_buffer[i] = sock->rx_buffer[i + to_read];
            }
        }
        sock->rx_len -= to_read;
    }
    
    return to_read;
}
