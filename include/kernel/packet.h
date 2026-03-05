#ifndef KERNEL_PACKET_H
#define KERNEL_PACKET_H

#include "../libc/stdint.h"
#include "netdev.h"

/* Process incoming packet from network device */
int packet_process(struct netdev *dev, struct net_packet *pkt);

/* Get packet processing statistics */
void packet_get_stats(uint32_t *received, uint32_t *dropped,
                      uint32_t *arp_pkt, uint32_t *ipv4_pkt);

/* Print packet statistics */
void packet_print_stats(void);

/* Reset statistics */
void packet_reset_stats(void);

#endif /* KERNEL_PACKET_H */
