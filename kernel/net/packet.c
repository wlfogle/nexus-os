#include "../../include/kernel/netdev.h"
#include "../../include/kernel/ethernet.h"
#include "../../include/kernel/arp.h"
#include "../../include/kernel/ipv4.h"
#include "../../include/kernel/serial.h"
#include <string.h>

/* Packet processing statistics */
static struct {
    uint32_t packets_received;
    uint32_t packets_dropped;
    uint32_t arp_received;
    uint32_t ipv4_received;
    uint32_t ethernet_errors;
} packet_stats = {0};

/* Process incoming packet from device */
int packet_process(struct netdev *dev, struct net_packet *pkt)
{
    if (!dev || !pkt || pkt->len < sizeof(struct eth_header)) {
        packet_stats.packets_dropped++;
        return -1;
    }

    packet_stats.packets_received++;

    /* Parse Ethernet frame */
    struct eth_header *eth_hdr = (struct eth_header *)pkt->data;
    uint16_t eth_type = ntohs(eth_hdr->type);

    switch (eth_type) {
        case ETH_TYPE_ARP:
            /* ARP packet */
            if (arp_receive(dev, pkt->data + sizeof(struct eth_header),
                           pkt->len - sizeof(struct eth_header)) == 0) {
                packet_stats.arp_received++;
                return 0;
            }
            break;

        case ETH_TYPE_IPv4:
            /* IPv4 packet */
            if (ipv4_receive(dev, pkt->data + sizeof(struct eth_header),
                            pkt->len - sizeof(struct eth_header)) == 0) {
                packet_stats.ipv4_received++;
                return 0;
            }
            break;

        default:
            /* Unknown type - silently drop */
            break;
    }

    packet_stats.packets_dropped++;
    return -1;
}

/* Get packet statistics */
void packet_get_stats(uint32_t *received, uint32_t *dropped,
                      uint32_t *arp_pkt, uint32_t *ipv4_pkt)
{
    if (received) *received = packet_stats.packets_received;
    if (dropped) *dropped = packet_stats.packets_dropped;
    if (arp_pkt) *arp_pkt = packet_stats.arp_received;
    if (ipv4_pkt) *ipv4_pkt = packet_stats.ipv4_received;
}

/* Print packet statistics */
void packet_print_stats(void)
{
    serial_printf("[Packet Stats] RX:%d Dropped:%d ARP:%d IPv4:%d\n",
                 packet_stats.packets_received,
                 packet_stats.packets_dropped,
                 packet_stats.arp_received,
                 packet_stats.ipv4_received);
}

/* Reset statistics */
void packet_reset_stats(void)
{
    memset(&packet_stats, 0, sizeof(packet_stats));
}
