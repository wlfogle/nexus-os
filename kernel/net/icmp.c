#include "../../include/kernel/icmp.h"
#include "../../include/kernel/ipv4.h"
#include "../../include/kernel/ethernet.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>

/* ICMP checksum calculation */
static uint16_t icmp_checksum(const void *data, uint32_t len)
{
    const uint16_t *words = (const uint16_t *)data;
    uint32_t sum = 0;
    
    /* Sum all 16-bit words */
    for (uint32_t i = 0; i < len / 2; i++) {
        sum += ntohs(words[i]);
    }
    
    /* Handle odd byte */
    if (len % 2) {
        sum += (((const uint8_t *)data)[len - 1]) << 8;
    }
    
    /* Add carry bits */
    sum = (sum & 0xFFFF) + (sum >> 16);
    sum = (sum & 0xFFFF) + (sum >> 16);
    
    /* Return one's complement */
    return htons(~sum & 0xFFFF);
}

/* Send ICMP echo request (ping) */
int icmp_send_echo(struct netdev *dev, const ipv4_addr_t *dest_ip,
                   uint16_t id, uint16_t seq,
                   const uint8_t *data, uint32_t len)
{
    if (!dev || !dest_ip || len > 1472) {
        return -1;
    }
    
    /* Allocate packet buffer */
    struct net_packet *pkt = netdev_alloc_packet();
    if (!pkt) return -1;
    
    /* Build ICMP echo request */
    struct icmp_header *hdr = (struct icmp_header *)pkt->data;
    
    hdr->type = ICMP_ECHO_REQUEST;
    hdr->code = 0;
    hdr->checksum = 0;
    hdr->identifier = htons(id);
    hdr->sequence = htons(seq);
    
    /* Copy payload */
    uint8_t *payload = pkt->data + sizeof(struct icmp_header);
    for (uint32_t i = 0; i < len; i++) {
        payload[i] = data[i];
    }
    
    uint32_t icmp_len = sizeof(struct icmp_header) + len;
    
    /* Compute ICMP checksum */
    hdr->checksum = icmp_checksum(hdr, icmp_len);
    
    pkt->len = icmp_len;
    
    /* Send via IPv4 */
    int ret = ipv4_send(dev, dest_ip, IPv4_PROTO_ICMP, pkt->data, icmp_len);
    
    netdev_free_packet(pkt);
    return ret;
}

/* Send ICMP echo reply */
static int icmp_send_reply(struct netdev *dev, const ipv4_addr_t *dest_ip,
                          uint16_t id, uint16_t seq,
                          const uint8_t *data, uint32_t len)
{
    if (!dev || !dest_ip || len > 1472) {
        return -1;
    }
    
    struct net_packet *pkt = netdev_alloc_packet();
    if (!pkt) return -1;
    
    struct icmp_header *hdr = (struct icmp_header *)pkt->data;
    
    hdr->type = ICMP_ECHO_REPLY;
    hdr->code = 0;
    hdr->checksum = 0;
    hdr->identifier = htons(id);
    hdr->sequence = htons(seq);
    
    /* Copy payload */
    uint8_t *payload = pkt->data + sizeof(struct icmp_header);
    for (uint32_t i = 0; i < len; i++) {
        payload[i] = data[i];
    }
    
    uint32_t icmp_len = sizeof(struct icmp_header) + len;
    
    /* Compute ICMP checksum */
    hdr->checksum = icmp_checksum(hdr, icmp_len);
    
    pkt->len = icmp_len;
    
    /* Send via IPv4 */
    int ret = ipv4_send(dev, dest_ip, IPv4_PROTO_ICMP, pkt->data, icmp_len);
    
    netdev_free_packet(pkt);
    return ret;
}

/* Handle incoming ICMP packet */
int icmp_receive(struct netdev *dev, const ipv4_addr_t *src_ip, const uint8_t *data, uint32_t len)
{
    if (!dev || !src_ip || !data || len < sizeof(struct icmp_header)) {
        return -1;
    }
    
    struct icmp_header *hdr = (struct icmp_header *)data;
    
    /* Verify checksum */
    uint16_t stored_checksum = hdr->checksum;
    ((struct icmp_header *)data)->checksum = 0;
    uint16_t calc_checksum = icmp_checksum(data, len);
    ((struct icmp_header *)data)->checksum = stored_checksum;
    
    if (stored_checksum != calc_checksum) {
        return -1;  /* Checksum mismatch */
    }
    
    uint8_t type = hdr->type;
    uint16_t id = ntohs(hdr->identifier);
    uint16_t seq = ntohs(hdr->sequence);
    
    /* Extract payload */
    uint8_t *payload = (uint8_t *)data + sizeof(struct icmp_header);
    uint32_t payload_len = len - sizeof(struct icmp_header);
    
    switch (type) {
        case ICMP_ECHO_REQUEST:
            /* Send echo reply to sender */
            serial_printf("[ICMP] Echo request (id=%d seq=%d)\n", id, seq);
            return icmp_send_reply(dev, src_ip, id, seq, payload, payload_len);
            
        case ICMP_ECHO_REPLY:
            serial_printf("[ICMP] Echo reply received (id=%d seq=%d)\n", id, seq);
            return 0;
            
        case ICMP_DEST_UNREACHABLE:
            serial_printf("[ICMP] Destination unreachable\n");
            return 0;
            
        case ICMP_TTL_EXCEEDED:
            serial_printf("[ICMP] TTL exceeded\n");
            return 0;
            
        default:
            return -1;  /* Unknown type */
    }
}
