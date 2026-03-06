#ifndef KERNEL_VIRTIO_NET_H
#define KERNEL_VIRTIO_NET_H

#include "../libc/stdint.h"
#include "netdev.h"

/* Initialize and register a virtio-net device */
int virtio_net_register(uint32_t io_base, const uint8_t *mac_addr);

/* Handle interrupt events */
void virtio_net_handle_rx(int device_id);
void virtio_net_handle_tx(int device_id);

/* Send/receive packets */
int virtio_net_send(struct netdev *dev, struct net_packet *pkt);
int virtio_net_receive(struct netdev *dev, struct net_packet *pkt);

/* Query device info */
int virtio_net_get_device_count(void);
int virtio_net_get_device_info(int device_id, uint8_t *mac_addr);

#endif /* KERNEL_VIRTIO_NET_H */
