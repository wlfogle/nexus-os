#include "../../include/kernel/arp.h"
#include "../../include/kernel/ethernet.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>

/* ARP cache table */
static struct arp_entry arp_cache[ARP_TABLE_SIZE];
static int arp_cache_count = 0;

/* Initialize ARP */
void arp_init(void)
{
    for (int i = 0; i < ARP_TABLE_SIZE; i++) {
        arp_cache[i].age = 0;
    }
    arp_cache_count = 0;
    serial_puts("[ARP] Cache initialized\n");
}

/* Add entry to ARP cache */
static void arp_cache_add(const ipv4_addr_t *ip, const mac_addr_t *mac)
{
    if (!ip || !mac) return;
    
    /* Check if already exists */
    for (int i = 0; i < arp_cache_count; i++) {
        if (ipv4_addr_equal(&arp_cache[i].ip, ip)) {
            eth_mac_copy(&arp_cache[i].mac, mac);
            arp_cache[i].age = 0;  /* Reset age */
            return;
        }
    }
    
    /* Add new entry if space available */
    if (arp_cache_count < ARP_TABLE_SIZE) {
        ipv4_addr_copy(&arp_cache[arp_cache_count].ip, ip);
        eth_mac_copy(&arp_cache[arp_cache_count].mac, mac);
        arp_cache[arp_cache_count].age = 0;
        arp_cache_count++;
    }
}

/* Look up ARP cache */
static int arp_cache_lookup(const ipv4_addr_t *ip, mac_addr_t *mac)
{
    if (!ip || !mac) return -1;
    
    for (int i = 0; i < arp_cache_count; i++) {
        if (ipv4_addr_equal(&arp_cache[i].ip, ip)) {
            eth_mac_copy(mac, &arp_cache[i].mac);
            return 0;  /* Found */
        }
    }
    
    return -1;  /* Not found */
}

/* Send ARP request */
int arp_request(struct netdev *dev, const ipv4_addr_t *target_ip)
{
    if (!dev || !target_ip) return -1;
    
    /* Allocate packet */
    struct net_packet *pkt = netdev_alloc_packet();
    if (!pkt) return -1;
    
    /* Build ARP packet */
    struct arp_packet *arp = (struct arp_packet *)pkt->data;
    
    arp->hw_type = htons(1);                    /* Ethernet */
    arp->proto_type = htons(ETH_TYPE_IPv4);     /* IPv4 */
    arp->hw_addr_len = ETH_ALEN;                /* 6 bytes */
    arp->proto_addr_len = 4;                    /* 4 bytes */
    arp->opcode = htons(ARP_OP_REQUEST);
    
    /* Sender hardware address (our MAC) */
    eth_mac_copy(&arp->sender_hw, &dev->mac_addr);
    
    /* Sender protocol address (our IP) */
    ipv4_addr_copy(&arp->sender_proto, &dev->ip_addr);
    
    /* Target hardware address (unknown, broadcast) */
    for (int i = 0; i < ETH_ALEN; i++) {
        arp->target_hw.addr[i] = 0;
    }
    
    /* Target protocol address */
    ipv4_addr_copy(&arp->target_proto, target_ip);
    
    pkt->len = sizeof(struct arp_packet);
    
    /* Send Ethernet frame with ARP payload */
    mac_addr_t broadcast;
    for (int i = 0; i < ETH_ALEN; i++) {
        broadcast.addr[i] = 0xFF;
    }
    
    int ret = eth_send(dev, &broadcast, ETH_TYPE_ARP, pkt->data, pkt->len);
    netdev_free_packet(pkt);
    
    return ret;
}

/* Send ARP reply */
static int arp_reply(struct netdev *dev, const mac_addr_t *dest_mac,
                     const ipv4_addr_t *target_ip, const mac_addr_t *target_mac)
{
    if (!dev || !dest_mac || !target_ip || !target_mac) return -1;
    
    struct net_packet *pkt = netdev_alloc_packet();
    if (!pkt) return -1;
    
    struct arp_packet *arp = (struct arp_packet *)pkt->data;
    
    arp->hw_type = htons(1);
    arp->proto_type = htons(ETH_TYPE_IPv4);
    arp->hw_addr_len = ETH_ALEN;
    arp->proto_addr_len = 4;
    arp->opcode = htons(ARP_OP_REPLY);
    
    /* Sender is us */
    eth_mac_copy(&arp->sender_hw, &dev->mac_addr);
    ipv4_addr_copy(&arp->sender_proto, &dev->ip_addr);
    
    /* Target is the requestor */
    eth_mac_copy(&arp->target_hw, target_mac);
    ipv4_addr_copy(&arp->target_proto, target_ip);
    
    pkt->len = sizeof(struct arp_packet);
    
    int ret = eth_send(dev, dest_mac, ETH_TYPE_ARP, pkt->data, pkt->len);
    netdev_free_packet(pkt);
    
    return ret;
}

/* Handle incoming ARP packet */
int arp_receive(struct netdev *dev, const uint8_t *data, uint32_t len)
{
    if (!dev || !data || len < sizeof(struct arp_packet)) {
        return -1;
    }
    
    struct arp_packet *arp = (struct arp_packet *)data;
    
    /* Validate ARP packet */
    if (ntohs(arp->hw_type) != 1 || ntohs(arp->proto_type) != ETH_TYPE_IPv4 ||
        arp->hw_addr_len != ETH_ALEN || arp->proto_addr_len != 4) {
        return -1;
    }
    
    /* Cache the sender's IP-MAC mapping */
    arp_cache_add(&arp->sender_proto, &arp->sender_hw);
    
    uint16_t opcode = ntohs(arp->opcode);
    
    switch (opcode) {
        case ARP_OP_REQUEST:
            /* Check if request is for us */
            if (ipv4_addr_equal(&arp->target_proto, &dev->ip_addr)) {
                /* Send reply */
                return arp_reply(dev, &arp->sender_hw, &arp->sender_proto, &arp->sender_hw);
            }
            break;
            
        case ARP_OP_REPLY:
            /* Just cache the entry (already done above) */
            serial_printf("[ARP] Cached IP "
                         "%d.%d.%d.%d -> "
                         "%02x:%02x:%02x:%02x:%02x:%02x\n",
                         arp->sender_proto.addr[0], arp->sender_proto.addr[1],
                         arp->sender_proto.addr[2], arp->sender_proto.addr[3],
                         arp->sender_hw.addr[0], arp->sender_hw.addr[1],
                         arp->sender_hw.addr[2], arp->sender_hw.addr[3],
                         arp->sender_hw.addr[4], arp->sender_hw.addr[5]);
            break;
    }
    
    return 0;
}

/* Resolve IP to MAC (with ARP if needed) */
int arp_resolve(struct netdev *dev, const ipv4_addr_t *ip, mac_addr_t *mac)
{
    if (!dev || !ip || !mac) return -1;
    
    /* Check if it's our own IP */
    if (ipv4_addr_equal(ip, &dev->ip_addr)) {
        eth_mac_copy(mac, &dev->mac_addr);
        return 0;
    }
    
    /* Check if it's broadcast */
    ipv4_addr_t broadcast = {{255, 255, 255, 255}};
    if (ipv4_addr_equal(ip, &broadcast)) {
        for (int i = 0; i < ETH_ALEN; i++) {
            mac->addr[i] = 0xFF;
        }
        return 0;
    }
    
    /* Try cache first */
    if (arp_cache_lookup(ip, mac) == 0) {
        return 0;  /* Found in cache */
    }
    
    /* Not in cache - would need to send ARP request and wait */
    /* For Phase 6, return error; Phase 7 can add blocking wait */
    return -1;
}

/* IPv4 address utilities */
int ipv4_addr_equal(const ipv4_addr_t *a, const ipv4_addr_t *b)
{
    if (!a || !b) return 0;
    for (int i = 0; i < 4; i++) {
        if (a->addr[i] != b->addr[i]) {
            return 0;
        }
    }
    return 1;
}

void ipv4_addr_copy(ipv4_addr_t *dest, const ipv4_addr_t *src)
{
    if (!dest || !src) return;
    for (int i = 0; i < 4; i++) {
        dest->addr[i] = src->addr[i];
    }
}

int ipv4_in_subnet(const ipv4_addr_t *ip, const ipv4_addr_t *network, const ipv4_addr_t *netmask)
{
    if (!ip || !network || !netmask) return 0;
    
    for (int i = 0; i < 4; i++) {
        if ((ip->addr[i] & netmask->addr[i]) != (network->addr[i] & netmask->addr[i])) {
            return 0;
        }
    }
    return 1;
}
