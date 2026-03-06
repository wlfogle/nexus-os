#include "../../include/kernel/netdev.h"
#include "../../include/kernel/packet.h"
#include "../../include/kernel/serial.h"
#include <string.h>

/* Virtio device configuration - QEMU standard */
#define VIRTIO_NET_DEVICE_ID    0x1000
#define VIRTIO_VENDOR_ID        0x1AF4
#define VIRTIO_NET_MAX_QUEUES   3

/* Virtio queue indices */
#define VIRTIO_NET_RQ           0  /* Receive queue */
#define VIRTIO_NET_SQ           1  /* Send queue */
#define VIRTIO_NET_CQ           2  /* Control queue */

/* Virtio net header */
struct virtio_net_hdr {
    uint8_t flags;
    uint8_t gso_type;
    uint16_t hdr_len;
    uint16_t gso_size;
    uint16_t csum_start;
    uint16_t csum_offset;
    uint16_t num_buffers;
};

/* Virtio net device state */
struct virtio_net_dev {
    uint32_t io_base;           /* PCI I/O base address */
    uint16_t mac[3];            /* MAC address (6 bytes) */
    uint32_t features;          /* Supported features */
    uint32_t active_queues;     /* Bitmask of active queues */
};

static struct virtio_net_dev virtio_devices[4];
static int virtio_device_count = 0;

/* Initialize virtio-net device */
int virtio_net_init(uint32_t io_base, const uint8_t *mac_addr)
{
    if (virtio_device_count >= 4) {
        serial_puts("[virtio-net] Max devices reached\n");
        return -1;
    }

    struct virtio_net_dev *dev = &virtio_devices[virtio_device_count];
    dev->io_base = io_base;
    dev->features = 0;
    dev->active_queues = 0;

    /* Copy MAC address */
    if (mac_addr) {
        for (int i = 0; i < 6; i += 2) {
            dev->mac[i/2] = (mac_addr[i] << 8) | mac_addr[i+1];
        }
    }

    serial_printf("[virtio-net] Device initialized at 0x%x\n", io_base);
    return virtio_device_count++;
}

/* Register virtio-net device with network stack */
int virtio_net_register(uint32_t io_base, const uint8_t *mac_addr)
{
    int dev_id = virtio_net_init(io_base, mac_addr);
    if (dev_id < 0) return -1;

    /* Create network device structure */
    struct netdev *netdev = kmalloc(sizeof(struct netdev));
    if (!netdev) return -1;

    netdev->dev_id = dev_id;
    netdev->name[0] = 'e';
    netdev->name[1] = 't';
    netdev->name[2] = 'h';
    netdev->name[3] = '0' + dev_id;
    netdev->name[4] = '\0';

    /* Set MAC address */
    if (mac_addr) {
        for (int i = 0; i < 6; i++) {
            netdev->mac_addr.addr[i] = mac_addr[i];
        }
    }

    /* Default IP (would be configured via DHCP or static config) */
    netdev->ip_addr.addr[0] = 192;
    netdev->ip_addr.addr[1] = 168;
    netdev->ip_addr.addr[2] = 1;
    netdev->ip_addr.addr[3] = 100 + dev_id;

    /* Network mask: 255.255.255.0 */
    netdev->netmask.addr[0] = 255;
    netdev->netmask.addr[1] = 255;
    netdev->netmask.addr[2] = 255;
    netdev->netmask.addr[3] = 0;

    /* Default gateway */
    netdev->gateway.addr[0] = 192;
    netdev->gateway.addr[1] = 168;
    netdev->gateway.addr[2] = 1;
    netdev->gateway.addr[3] = 1;

    netdev->mtu = 1500;
    netdev->flags = IFF_UP | IFF_RUNNING;
    netdev->ops = NULL;  /* Would point to virtio_net_ops in full implementation */
    netdev->priv = (void *)(uintptr_t)dev_id;

    serial_printf("[virtio-net] Registering eth%d with IP %d.%d.%d.%d\n",
                 dev_id,
                 netdev->ip_addr.addr[0],
                 netdev->ip_addr.addr[1],
                 netdev->ip_addr.addr[2],
                 netdev->ip_addr.addr[3]);

    return netdev_register(netdev);
}

/* Handle virtio-net RX interrupt - called by IRQ handler */
void virtio_net_handle_rx(int device_id)
{
    if (device_id < 0 || device_id >= virtio_device_count) return;

    struct virtio_net_dev *dev = &virtio_devices[device_id];
    struct netdev *netdev = netdev_get(device_id);

    if (!netdev) return;

    /* In a full implementation:
     * 1. Read packet from virtio RX ring
     * 2. Remove virtio header
     * 3. Allocate packet buffer
     * 4. Call packet_process(netdev, pkt)
     * 5. Return descriptor to device
     *
     * For now, this is a placeholder showing the flow.
     */

    serial_printf("[virtio-net] RX interrupt on eth%d\n", device_id);
}

/* Handle virtio-net TX completion - called by IRQ handler */
void virtio_net_handle_tx(int device_id)
{
    if (device_id < 0 || device_id >= virtio_device_count) return;

    /* In a full implementation:
     * 1. Check TX ring for completed packets
     * 2. Free buffer descriptors
     * 3. Update statistics
     *
     * For now, this is a placeholder.
     */

    serial_printf("[virtio-net] TX complete on eth%d\n", device_id);
}

/* Send packet through virtio-net device */
int virtio_net_send(struct netdev *dev, struct net_packet *pkt)
{
    if (!dev || !pkt) return -1;

    int device_id = (uintptr_t)dev->priv;
    if (device_id < 0 || device_id >= virtio_device_count) return -1;

    /* In a full implementation:
     * 1. Add virtio header to packet
     * 2. Add packet to virtio TX ring
     * 3. Notify device
     * 4. Wait for completion or return immediately
     *
     * For now, just return success as placeholder.
     */

    return pkt->len;
}

/* Receive packet from virtio-net device */
int virtio_net_receive(struct netdev *dev, struct net_packet *pkt)
{
    if (!dev || !pkt) return -1;

    /* This would be called by the device interrupt handler
     * to process received packets through the dispatcher.
     *
     * In a full implementation, packets come from the
     * virtio RX ring via handle_rx.
     */

    return packet_process(dev, pkt);
}

/* Get device count */
int virtio_net_get_device_count(void)
{
    return virtio_device_count;
}

/* Get device info */
int virtio_net_get_device_info(int device_id, uint8_t *mac_addr)
{
    if (device_id < 0 || device_id >= virtio_device_count) return -1;
    if (!mac_addr) return -1;

    struct virtio_net_dev *dev = &virtio_devices[device_id];

    /* Copy MAC address */
    for (int i = 0; i < 6; i += 2) {
        mac_addr[i] = (dev->mac[i/2] >> 8) & 0xFF;
        mac_addr[i+1] = dev->mac[i/2] & 0xFF;
    }

    return 0;
}
