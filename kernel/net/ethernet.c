#include "../../include/kernel/ethernet.h"
#include "../../include/kernel/serial.h"
#include "../../include/kernel/arp.h"
#include "../../include/kernel/ipv4.h"
#include <stddef.h>

/* Send Ethernet frame */
int eth_send(struct netdev *dev, const mac_addr_t *dest_mac, uint16_t type,
             const uint8_t *payload, uint32_t len)
{
    if (!dev || !dest_mac || !payload || len == 0 || len > MTU) {
        return -1;
    }
    
    /* Allocate packet */
    struct net_packet *pkt = netdev_alloc_packet();
    if (!pkt) return -1;
    
    /* Build Ethernet frame */
    struct eth_header *hdr = (struct eth_header *)pkt->data;
    
    /* Destination MAC */
    for (int i = 0; i < ETH_ALEN; i++) {
        hdr->dest_mac[i] = dest_mac->addr[i];
    }
    
    /* Source MAC */
    for (int i = 0; i < ETH_ALEN; i++) {
        hdr->src_mac[i] = dev->mac_addr.addr[i];
    }
    
    /* EtherType */
    hdr->type = htons(type);
    
    /* Payload */
    uint8_t *payload_ptr = pkt->data + sizeof(struct eth_header);
    for (uint32_t i = 0; i < len; i++) {
        payload_ptr[i] = payload[i];
    }
    
    /* Set packet length */
    pkt->len = sizeof(struct eth_header) + len;
    
    /* Send via device */
    int ret = netdev_send(dev, pkt);
    
    /* Free packet */
    netdev_free_packet(pkt);
    
    return ret;
}

/* Handle incoming Ethernet frame */
int eth_receive(struct netdev *dev, struct net_packet *pkt)
{
    if (!dev || !pkt || pkt->len < sizeof(struct eth_header)) {
        return -1;
    }
    
    struct eth_header *hdr = (struct eth_header *)pkt->data;
    uint16_t type = ntohs(hdr->type);
    
    /* Skip Ethernet header */
    pkt->offset = sizeof(struct eth_header);
    uint8_t *payload = pkt->data + pkt->offset;
    uint32_t payload_len = pkt->len - pkt->offset;
    
    /* Route to appropriate protocol handler */
    switch (type) {
        case ETH_TYPE_ARP:
            return arp_receive(dev, payload, payload_len);
        case ETH_TYPE_IPv4:
            return ipv4_receive(dev, payload, payload_len);
        default:
            return -1;  /* Unknown protocol */
    }
}

/* MAC address comparison */
int eth_mac_equal(const mac_addr_t *a, const mac_addr_t *b)
{
    if (!a || !b) return 0;
    for (int i = 0; i < ETH_ALEN; i++) {
        if (a->addr[i] != b->addr[i]) {
            return 0;
        }
    }
    return 1;
}

/* Copy MAC address */
void eth_mac_copy(mac_addr_t *dest, const mac_addr_t *src)
{
    if (!dest || !src) return;
    for (int i = 0; i < ETH_ALEN; i++) {
        dest->addr[i] = src->addr[i];
    }
}

/* Check if broadcast MAC (FF:FF:FF:FF:FF:FF) */
int eth_is_broadcast(const mac_addr_t *mac)
{
    if (!mac) return 0;
    for (int i = 0; i < ETH_ALEN; i++) {
        if (mac->addr[i] != 0xFF) {
            return 0;
        }
    }
    return 1;
}

/* Check if multicast MAC (first octet LSB = 1) */
int eth_is_multicast(const mac_addr_t *mac)
{
    if (!mac) return 0;
    return (mac->addr[0] & 0x01) == 1;
}
