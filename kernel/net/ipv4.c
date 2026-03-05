#include "../../include/kernel/ipv4.h"
#include "../../include/kernel/ethernet.h"
#include "../../include/kernel/arp.h"
#include "../../include/kernel/icmp.h"
#include "../../include/kernel/udp.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>

/* IPv4 checksum calculation */
static uint16_t ipv4_checksum(const void *data, uint32_t len)
{
    const uint16_t *words = (const uint16_t *)data;
    uint32_t sum = 0;
    
    /* Sum all 16-bit words */
    for (uint32_t i = 0; i < len / 2; i++) {
        sum += ntohs(words[i]);
    }
    
    /* Add carry bits */
    sum = (sum & 0xFFFF) + (sum >> 16);
    sum = (sum & 0xFFFF) + (sum >> 16);
    
    /* Return one's complement */
    return htons(~sum & 0xFFFF);
}

/* Handle incoming IPv4 packet */
int ipv4_receive(struct netdev *dev, const uint8_t *data, uint32_t len)
{
    if (!dev || !data || len < sizeof(struct ipv4_header)) {
        return -1;
    }
    
    struct ipv4_header *hdr = (struct ipv4_header *)data;
    
    /* Verify version and header length */
    uint8_t version = (hdr->version_ihl >> 4) & 0x0F;
    uint8_t ihl = (hdr->version_ihl & 0x0F) * 4;  /* In bytes */
    
    if (version != 4 || ihl < 20) {
        return -1;
    }
    
    /* Verify header checksum */
    uint16_t stored_checksum = hdr->checksum;
    hdr->checksum = 0;
    uint16_t calc_checksum = ipv4_checksum(hdr, ihl);
    hdr->checksum = stored_checksum;
    
    if (stored_checksum != calc_checksum) {
        return -1;  /* Checksum mismatch */
    }
    
    /* Check if packet is for us */
    if (!ipv4_addr_equal(&hdr->dest_ip, &dev->ip_addr)) {
        return -1;  /* Not for us */
    }
    
    uint8_t protocol = hdr->protocol;
    uint8_t *payload = (uint8_t *)hdr + ihl;
    uint32_t payload_len = ntohs(hdr->total_length) - ihl;
    
    /* Route to protocol handler */
    switch (protocol) {
        case IPv4_PROTO_ICMP:
            return icmp_receive(dev, &hdr->src_ip, payload, payload_len);
            
        case IPv4_PROTO_UDP:
            return udp_receive(dev, &hdr->src_ip, payload, payload_len);
            
        case IPv4_PROTO_TCP:
            /* TCP handler would go here (Phase 6.6) */
            serial_puts("[IPv4] TCP packet received (handler not implemented)\n");
            return 0;
            
        default:
            return -1;  /* Unknown protocol */
    }
}

/* Send IPv4 packet */
int ipv4_send(struct netdev *dev, const ipv4_addr_t *dest_ip, uint8_t protocol,
              const uint8_t *payload, uint32_t len)
{
    if (!dev || !dest_ip || !payload || len == 0) {
        return -1;
    }
    
    if (len > (MTU - sizeof(struct ipv4_header))) {
        return -1;  /* Payload too large */
    }
    
    /* Allocate packet */
    struct net_packet *pkt = netdev_alloc_packet();
    if (!pkt) return -1;
    
    /* Build IPv4 header */
    struct ipv4_header *hdr = (struct ipv4_header *)pkt->data;
    
    hdr->version_ihl = (4 << 4) | 5;        /* Version 4, IHL 5 (20 bytes) */
    hdr->dscp_ecn = 0;
    hdr->total_length = htons(sizeof(struct ipv4_header) + len);
    hdr->identification = 0;
    hdr->flags_offset = 0;                  /* No fragmentation */
    hdr->ttl = 64;
    hdr->protocol = protocol;
    hdr->checksum = 0;                      /* Will compute below */
    
    ipv4_addr_copy(&hdr->src_ip, &dev->ip_addr);
    ipv4_addr_copy(&hdr->dest_ip, dest_ip);
    
    /* Compute header checksum */
    hdr->checksum = ipv4_checksum(hdr, sizeof(struct ipv4_header));
    
    /* Copy payload */
    uint8_t *pkt_payload = pkt->data + sizeof(struct ipv4_header);
    for (uint32_t i = 0; i < len; i++) {
        pkt_payload[i] = payload[i];
    }
    
    pkt->len = sizeof(struct ipv4_header) + len;
    
    /* Resolve destination MAC via ARP */
    mac_addr_t dest_mac;
    if (arp_resolve(dev, dest_ip, &dest_mac) != 0) {
        /* Could not resolve - send ARP request */
        arp_request(dev, dest_ip);
        netdev_free_packet(pkt);
        return -1;
    }
    
    /* Send Ethernet frame */
    int ret = eth_send(dev, &dest_mac, ETH_TYPE_IPv4, pkt->data, pkt->len);
    netdev_free_packet(pkt);
    
    return ret;
}

/* IPv4 utilities - forward declarations from arp.c */
/* These are defined in arp.c and declared in arp.h */
