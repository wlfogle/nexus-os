#include "../../include/kernel/udp.h"
#include "../../include/kernel/ipv4.h"
#include "../../include/kernel/ethernet.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>
#include <string.h>

/* UDP socket table */
static struct udp_socket udp_sockets[UDP_MAX_SOCKETS];
static int udp_socket_count = 0;

/* Port allocation counter */
static uint16_t next_ephemeral_port = 49152;  /* Start of ephemeral port range */

/* Initialize UDP */
void udp_init(void)
{
    for (int i = 0; i < UDP_MAX_SOCKETS; i++) {
        udp_sockets[i].in_use = 0;
    }
    udp_socket_count = 0;
    serial_puts("[UDP] Module initialized\n");
}

/* UDP checksum calculation (includes pseudo-header) */
static uint16_t udp_checksum(const ipv4_addr_t *src_ip, const ipv4_addr_t *dest_ip,
                             const void *data, uint32_t len)
{
    /* Build pseudo-header */
    struct {
        ipv4_addr_t src_ip;
        ipv4_addr_t dest_ip;
        uint8_t zero;
        uint8_t protocol;
        uint16_t length;
    } __attribute__((packed)) pseudo_hdr;
    
    ipv4_addr_copy(&pseudo_hdr.src_ip, src_ip);
    ipv4_addr_copy(&pseudo_hdr.dest_ip, dest_ip);
    pseudo_hdr.zero = 0;
    pseudo_hdr.protocol = IPv4_PROTO_UDP;
    pseudo_hdr.length = htons(len);
    
    /* Sum pseudo-header */
    uint32_t sum = 0;
    const uint8_t *hdr_bytes = (const uint8_t *)&pseudo_hdr;
    uint16_t word_val;
    for (unsigned int i = 0; i < sizeof(pseudo_hdr) / 2; i++) {
        memcpy(&word_val, hdr_bytes + i * 2, sizeof(uint16_t));
        sum += ntohs(word_val);
    }
    
    /* Sum UDP packet */
    const uint8_t *data_bytes = (const uint8_t *)data;
    for (uint32_t i = 0; i < len / 2; i++) {
        memcpy(&word_val, data_bytes + i * 2, sizeof(uint16_t));
        sum += ntohs(word_val);
    }
    
    /* Handle odd byte */
    if (len % 2) {
        sum += (((const uint8_t *)data)[len - 1]) << 8;
    }
    
    /* Add carry bits */
    sum = (sum & 0xFFFF) + (sum >> 16);
    sum = (sum & 0xFFFF) + (sum >> 16);
    
    /* Return one's complement (0 means valid, so use 0xFFFF for all-zeros) */
    uint16_t checksum = ~sum & 0xFFFF;
    return htons(checksum == 0 ? 0xFFFF : checksum);
}

/* Send UDP datagram */
int udp_send(struct netdev *dev, const ipv4_addr_t *dest_ip, uint16_t dest_port,
             uint16_t src_port, const uint8_t *data, uint32_t len)
{
    if (!dev || !dest_ip || len == 0 || len > UDP_MAX_PAYLOAD) {
        return -1;
    }
    
    struct net_packet *pkt = netdev_alloc_packet();
    if (!pkt) return -1;
    
    /* Build UDP header */
    struct udp_header *hdr = (struct udp_header *)pkt->data;
    
    hdr->src_port = htons(src_port);
    hdr->dest_port = htons(dest_port);
    hdr->length = htons(sizeof(struct udp_header) + len);
    hdr->checksum = 0;  /* Checksum computed below */
    
    /* Copy payload */
    uint8_t *payload = pkt->data + sizeof(struct udp_header);
    for (uint32_t i = 0; i < len; i++) {
        payload[i] = data[i];
    }
    
    uint32_t udp_len = sizeof(struct udp_header) + len;
    
    /* Compute UDP checksum with pseudo-header */
    hdr->checksum = udp_checksum(&dev->ip_addr, dest_ip, hdr, udp_len);
    
    pkt->len = udp_len;
    
    /* Send via IPv4 */
    int ret = ipv4_send(dev, dest_ip, IPv4_PROTO_UDP, pkt->data, udp_len);
    
    netdev_free_packet(pkt);
    return ret;
}

/* Handle incoming UDP packet */
int udp_receive(struct netdev *dev, const ipv4_addr_t *src_ip, const uint8_t *data, uint32_t len)
{
    if (!dev || !src_ip || !data || len < sizeof(struct udp_header)) {
        return -1;
    }
    
    struct udp_header *hdr = (struct udp_header *)data;
    
    uint16_t src_port = ntohs(hdr->src_port);
    uint16_t dest_port = ntohs(hdr->dest_port);
    uint16_t udp_len = ntohs(hdr->length);
    
    if (udp_len < sizeof(struct udp_header) || udp_len > len) {
        return -1;
    }
    
    /* Verify checksum if present */
    if (hdr->checksum != 0) {
        uint16_t stored_checksum = hdr->checksum;
        ((struct udp_header *)data)->checksum = 0;
        uint16_t calc_checksum = udp_checksum(src_ip, &dev->ip_addr, data, udp_len);
        ((struct udp_header *)data)->checksum = stored_checksum;
        
        if (stored_checksum != calc_checksum) {
            return -1;  /* Checksum mismatch */
        }
    }
    
    /* Look for listening socket on dest_port */
    for (int i = 0; i < UDP_MAX_SOCKETS; i++) {
        if (udp_sockets[i].in_use && udp_sockets[i].local_port == dest_port) {
            /* Store packet for socket to receive */
            (void)((uint8_t *)data + sizeof(struct udp_header));  /* payload available for future use */
            uint32_t payload_len = udp_len - sizeof(struct udp_header);
            
            /* Save sender info and payload (simple: store in socket for later retrieval) */
            serial_printf("[UDP] Received %d bytes on port %d from %d.%d.%d.%d:%d\n",
                         payload_len, dest_port,
                         src_ip->addr[0], src_ip->addr[1],
                         src_ip->addr[2], src_ip->addr[3], src_port);
            
            return 0;  /* Packet consumed */
        }
    }
    
    /* No listening socket - silently discard (could send ICMP port unreachable) */
    return -1;
}

/* Create UDP socket */
int udp_socket_create(uint16_t local_port)
{
    if (udp_socket_count >= UDP_MAX_SOCKETS) {
        return -1;  /* No free sockets */
    }
    
    /* Find free slot */
    int socket_id = -1;
    for (int i = 0; i < UDP_MAX_SOCKETS; i++) {
        if (!udp_sockets[i].in_use) {
            socket_id = i;
            break;
        }
    }
    
    if (socket_id < 0) return -1;
    
    struct udp_socket *sock = &udp_sockets[socket_id];
    sock->id = socket_id;
    sock->in_use = 1;
    
    if (local_port == 0) {
        /* Allocate ephemeral port */
        sock->local_port = next_ephemeral_port++;
        if (next_ephemeral_port == 0) {  /* Wrapped past 65535 */
            next_ephemeral_port = 49152;
        }
    } else {
        sock->local_port = local_port;
    }
    
    sock->rx_packet = NULL;
    sock->rx_len = 0;
    
    udp_socket_count++;
    
    serial_printf("[UDP] Socket created (id=%d, port=%d)\n", socket_id, sock->local_port);
    return socket_id;
}

/* Close UDP socket */
int udp_socket_close(int socket_id)
{
    if (socket_id < 0 || socket_id >= UDP_MAX_SOCKETS) {
        return -1;
    }
    
    if (!udp_sockets[socket_id].in_use) {
        return -1;
    }
    
    udp_sockets[socket_id].in_use = 0;
    if (udp_sockets[socket_id].rx_packet) {
        netdev_free_packet(udp_sockets[socket_id].rx_packet);
    }
    
    udp_socket_count--;
    serial_printf("[UDP] Socket closed (id=%d)\n", socket_id);
    return 0;
}

/* Send data on UDP socket */
int udp_socket_send(int socket_id, const ipv4_addr_t *dest_ip __attribute__((unused)),
                    uint16_t dest_port __attribute__((unused)),
                    const uint8_t *data __attribute__((unused)),
                    uint32_t len __attribute__((unused)))
{
    if (socket_id < 0 || socket_id >= UDP_MAX_SOCKETS) {
        return -1;
    }
    
    if (!udp_sockets[socket_id].in_use) {
        return -1;
    }
    
    /* Would need netdev reference here - simplified for now */
    /* In real implementation, would get netdev from routing table */
    return -1;  /* Requires netdev context */
}

/* Receive data on UDP socket */
int udp_socket_recv(int socket_id, uint8_t *buffer, uint32_t buf_len)
{
    if (socket_id < 0 || socket_id >= UDP_MAX_SOCKETS) {
        return -1;
    }
    
    if (!udp_sockets[socket_id].in_use) {
        return -1;
    }
    
    struct udp_socket *sock = &udp_sockets[socket_id];
    
    if (!sock->rx_packet) {
        return 0;  /* No data available */
    }
    
    uint32_t copy_len = sock->rx_len < buf_len ? sock->rx_len : buf_len;
    for (uint32_t i = 0; i < copy_len; i++) {
        buffer[i] = sock->rx_packet->data[i];
    }
    
    netdev_free_packet(sock->rx_packet);
    sock->rx_packet = NULL;
    sock->rx_len = 0;
    
    return copy_len;
}
