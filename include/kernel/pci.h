#ifndef KERNEL_PCI_H
#define KERNEL_PCI_H

#include "../libc/stdint.h"

/* PCI Configuration Space Register Offsets */
#define PCI_REG_VENDOR_ID       0x00
#define PCI_REG_DEVICE_ID       0x02
#define PCI_REG_COMMAND         0x04
#define PCI_REG_STATUS          0x06
#define PCI_REG_REVISION_ID     0x08
#define PCI_REG_CLASS_CODE      0x09
#define PCI_REG_CACHE_LINE_SIZE 0x0C
#define PCI_REG_LATENCY_TIMER   0x0D
#define PCI_REG_HEADER_TYPE     0x0E
#define PCI_REG_BIST            0x0F
#define PCI_REG_BAR0            0x10
#define PCI_REG_BAR1            0x14
#define PCI_REG_BAR2            0x18
#define PCI_REG_BAR3            0x1C
#define PCI_REG_BAR4            0x20
#define PCI_REG_BAR5            0x24
#define PCI_REG_IRQ_LINE        0x3C
#define PCI_REG_IRQ_PIN         0x3D

/* PCI Command Register Flags */
#define PCI_CMD_IO_SPACE        0x0001
#define PCI_CMD_MEMORY_SPACE    0x0002
#define PCI_CMD_BUS_MASTER      0x0004
#define PCI_CMD_INTERRUPT       0x0400

/* PCI Header Types */
#define PCI_HEADER_DEVICE       0x00
#define PCI_HEADER_BRIDGE       0x01

/* PCI Device Class Codes */
#define PCI_CLASS_UNCLASSIFIED  0x00
#define PCI_CLASS_MASS_STORAGE  0x01
#define PCI_CLASS_NETWORK       0x02
#define PCI_CLASS_DISPLAY       0x03
#define PCI_CLASS_MULTIMEDIA    0x04
#define PCI_CLASS_MEMORY        0x05
#define PCI_CLASS_BRIDGE        0x06
#define PCI_CLASS_COMM          0x07
#define PCI_CLASS_GENERIC       0x08
#define PCI_CLASS_INPUT         0x09

/* PCI BAR Type */
#define PCI_BAR_IO              0x01
#define PCI_BAR_MEMORY          0x00
#define PCI_BAR_MEMORY_64BIT    0x04

/* PCI Device Structure */
typedef struct {
    uint16_t vendor_id;
    uint16_t device_id;
    uint16_t command;
    uint16_t status;
    uint8_t revision_id;
    uint8_t class_code;
    uint8_t subclass_code;
    uint8_t interface_code;
    uint8_t header_type;
    uint8_t irq_line;
    uint8_t irq_pin;
    uint32_t bars[6];           /* Base Address Registers */
    uint32_t bus;               /* PCI Bus number */
    uint32_t slot;              /* Slot number */
    uint32_t function;          /* Function number */
    uint32_t pci_device_id;     /* Unique PCI device ID */
} pci_device_t;

/* PCI Bus Structure */
typedef struct {
    uint8_t bus_num;
    uint32_t device_count;
    pci_device_t devices[32];   /* Max 32 devices per bus */
} pci_bus_t;

/* PCI Configuration Space Access */
uint8_t pci_config_read8(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset);
uint16_t pci_config_read16(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset);
uint32_t pci_config_read32(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset);

void pci_config_write8(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset, uint8_t val);
void pci_config_write16(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset, uint16_t val);
void pci_config_write32(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset, uint32_t val);

/* PCI Enumeration */
void pci_init(void);
int pci_enumerate(void);
pci_device_t *pci_find_device(uint16_t vendor_id, uint16_t device_id);
pci_device_t *pci_find_device_class(uint8_t class_code, uint8_t subclass);
pci_device_t *pci_get_device(int index);
int pci_device_count(void);

/* PCI Device Operations */
void pci_enable_device(pci_device_t *dev);
void pci_disable_device(pci_device_t *dev);
uint32_t pci_get_bar_address(pci_device_t *dev, int bar_num);
uint32_t pci_get_bar_size(pci_device_t *dev, int bar_num);

#endif /* KERNEL_PCI_H */
