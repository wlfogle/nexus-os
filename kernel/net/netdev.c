#include "../../include/kernel/netdev.h"
#include "../../include/kernel/heap.h"
#include "../../include/kernel/serial.h"
#include <stddef.h>

/* Global device table */
static struct netdev *netdev_table[MAX_NETDEVS];
static int netdev_count = 0;

/* Forward declaration */
static int string_equal(const char *a, const char *b);

/* Packet buffer pool (32 buffers) */
#define PACKET_POOL_SIZE 32
static struct net_packet packet_pool[PACKET_POOL_SIZE];
static uint8_t packet_pool_alloc[PACKET_POOL_SIZE];

/* Initialize networking */
void netdev_init(void)
{
    for (int i = 0; i < MAX_NETDEVS; i++) {
        netdev_table[i] = NULL;
    }
    netdev_count = 0;
    
    /* Initialize packet pool */
    for (int i = 0; i < PACKET_POOL_SIZE; i++) {
        packet_pool_alloc[i] = 0;
    }
    
    serial_puts("[NET] Device layer initialized\n");
}

/* Register network device */
int netdev_register(struct netdev *dev)
{
    if (!dev || netdev_count >= MAX_NETDEVS) {
        return -1;
    }
    
    dev->dev_id = netdev_count;
    netdev_table[netdev_count] = dev;
    netdev_count++;
    
    serial_printf("[NET] Registered device: %s (id=%d)\n", dev->name, dev->dev_id);
    return dev->dev_id;
}

/* Unregister network device */
void netdev_unregister(struct netdev *dev)
{
    if (!dev) return;
    
    for (int i = 0; i < MAX_NETDEVS; i++) {
        if (netdev_table[i] == dev) {
            netdev_table[i] = NULL;
            serial_printf("[NET] Unregistered device: %s\n", dev->name);
            return;
        }
    }
}

/* Get device by ID */
struct netdev *netdev_get(uint32_t dev_id)
{
    if (dev_id < MAX_NETDEVS) {
        return netdev_table[dev_id];
    }
    return NULL;
}

/* Get device by name */
struct netdev *netdev_get_by_name(const char *name)
{
    for (int i = 0; i < MAX_NETDEVS; i++) {
        if (netdev_table[i] && string_equal(netdev_table[i]->name, name)) {
            return netdev_table[i];
        }
    }
    return NULL;
}

/* Allocate packet from pool */
struct net_packet *netdev_alloc_packet(void)
{
    for (int i = 0; i < PACKET_POOL_SIZE; i++) {
        if (!packet_pool_alloc[i]) {
            packet_pool_alloc[i] = 1;
            struct net_packet *pkt = &packet_pool[i];
            pkt->len = 0;
            pkt->offset = 0;
            pkt->flags = 0;
            return pkt;
        }
    }
    return NULL;  /* No free packets */
}

/* Free packet back to pool */
void netdev_free_packet(struct net_packet *pkt)
{
    if (!pkt) return;
    
    for (int i = 0; i < PACKET_POOL_SIZE; i++) {
        if (&packet_pool[i] == pkt) {
            packet_pool_alloc[i] = 0;
            return;
        }
    }
}

/* Send packet on device */
int netdev_send(struct netdev *dev, struct net_packet *pkt)
{
    if (!dev || !pkt || !dev->ops || !dev->ops->send) {
        return -1;
    }
    
    if (!(dev->flags & IFF_UP)) {
        return -1;  /* Device not up */
    }
    
    return dev->ops->send(dev, pkt);
}

/* Receive packet from device */
int netdev_receive(struct netdev *dev, struct net_packet *pkt)
{
    if (!dev || !pkt || !dev->ops || !dev->ops->receive) {
        return -1;
    }
    
    if (!(dev->flags & IFF_UP)) {
        return -1;  /* Device not up */
    }
    
    return dev->ops->receive(dev, pkt);
}

/* Utility function - string comparison (for netdev_get_by_name) */
static int string_equal(const char *a, const char *b)
{
    if (!a || !b) return 0;
    while (*a && *b && *a == *b) {
        a++;
        b++;
    }
    return *a == *b;
}
