#ifndef KERNEL_NETDEV_H
#define KERNEL_NETDEV_H

#include "../libc/stdint.h"

#define MAX_NETDEVS 4
#define MTU 1500
#define ETH_ALEN 6

/* MAC address */
typedef struct {
    uint8_t addr[ETH_ALEN];
} mac_addr_t;

/* IPv4 address */
typedef struct {
    uint8_t addr[4];
} ipv4_addr_t;

/* Network packet buffer */
struct net_packet {
    uint8_t data[MTU + 100];  /* Extra for headers */
    uint32_t len;              /* Data length */
    uint32_t offset;           /* Current offset for processing */
    uint32_t flags;            /* RX/TX flags */
};

/* Forward declaration for use in netdev_ops */
struct netdev;

/* Network device operations */
struct netdev_ops {
    int (*send)(struct netdev *dev, struct net_packet *pkt);
    int (*receive)(struct netdev *dev, struct net_packet *pkt);
    int (*set_address)(struct netdev *dev, const mac_addr_t *mac);
};

/* Network device */
struct netdev {
    uint32_t dev_id;
    char name[16];
    mac_addr_t mac_addr;
    ipv4_addr_t ip_addr;
    ipv4_addr_t gateway;
    ipv4_addr_t netmask;
    uint32_t mtu;
    uint32_t flags;  /* IFF_UP, IFF_LOOPBACK, etc */
    struct netdev_ops *ops;
    void *priv;      /* Driver-specific data */
};

/* Device flags */
#define IFF_UP        0x01
#define IFF_LOOPBACK  0x02
#define IFF_RUNNING   0x04

/* Register/unregister network devices */
int netdev_register(struct netdev *dev);
void netdev_unregister(struct netdev *dev);

/* Get device by ID or name */
struct netdev *netdev_get(uint32_t dev_id);
struct netdev *netdev_get_by_name(const char *name);

/* Packet operations */
struct net_packet *netdev_alloc_packet(void);
void netdev_free_packet(struct net_packet *pkt);

/* Send/receive on device */
int netdev_send(struct netdev *dev, struct net_packet *pkt);
int netdev_receive(struct netdev *dev, struct net_packet *pkt);

#endif /* KERNEL_NETDEV_H */
