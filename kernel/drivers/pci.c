#include "../../include/kernel/pci.h"
#include "../../include/kernel/serial.h"
#include <string.h>

/* PCI Configuration Space Access I/O Ports (Type 1) */
#define PCI_CONFIG_ADDRESS  0xCF8
#define PCI_CONFIG_DATA     0xCFC

#define MAX_PCI_DEVICES 256

static pci_device_t pci_devices[MAX_PCI_DEVICES];
static int pci_device_count_total = 0;

/* I/O port operations */
static inline void outl(uint16_t port, uint32_t val)
{
    __asm__ volatile("outl %0, %1" : : "a"(val), "Nd"(port));
}

static inline void outw(uint16_t port, uint16_t val)
{
    __asm__ volatile("outw %0, %1" : : "a"(val), "Nd"(port));
}

static inline void outb(uint16_t port, uint8_t val)
{
    __asm__ volatile("outb %0, %1" : : "a"(val), "Nd"(port));
}

static inline uint32_t inl(uint16_t port)
{
    uint32_t ret;
    __asm__ volatile("inl %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static inline uint16_t inw(uint16_t port)
{
    uint16_t ret;
    __asm__ volatile("inw %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static inline uint8_t inb(uint16_t port)
{
    uint8_t ret;
    __asm__ volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

void pci_init(void)
{
    memset(pci_devices, 0, sizeof(pci_devices));
    pci_device_count_total = 0;
    
    serial_puts("[pci] PCI subsystem initialized\n");
}

uint8_t pci_config_read8(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset)
{
    uint32_t address = (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xFC) | 0x80000000;
    outl(PCI_CONFIG_ADDRESS, address);
    uint8_t val = inb(PCI_CONFIG_DATA + (offset & 3));
    return val;
}

uint16_t pci_config_read16(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset)
{
    uint32_t address = (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xFC) | 0x80000000;
    outl(PCI_CONFIG_ADDRESS, address);
    uint16_t val = inw(PCI_CONFIG_DATA + (offset & 2));
    return val;
}

uint32_t pci_config_read32(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset)
{
    uint32_t address = (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xFC) | 0x80000000;
    outl(PCI_CONFIG_ADDRESS, address);
    uint32_t val = inl(PCI_CONFIG_DATA);
    return val;
}

void pci_config_write8(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset, uint8_t val)
{
    uint32_t address = (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xFC) | 0x80000000;
    outl(PCI_CONFIG_ADDRESS, address);
    outb(PCI_CONFIG_DATA + (offset & 3), val);
}

void pci_config_write16(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset, uint16_t val)
{
    uint32_t address = (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xFC) | 0x80000000;
    outl(PCI_CONFIG_ADDRESS, address);
    outw(PCI_CONFIG_DATA + (offset & 2), val);
}

void pci_config_write32(uint32_t bus, uint32_t slot, uint32_t func, uint32_t offset, uint32_t val)
{
    uint32_t address = (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xFC) | 0x80000000;
    outl(PCI_CONFIG_ADDRESS, address);
    outl(PCI_CONFIG_DATA, val);
}

int pci_enumerate(void)
{
    int found = 0;
    
    /* Scan PCI bus 0 for devices */
    for (uint32_t slot = 0; slot < 32; slot++) {
        for (uint32_t func = 0; func < 8; func++) {
            uint16_t vendor_id = pci_config_read16(0, slot, func, PCI_REG_VENDOR_ID);
            
            /* Check if device exists (vendor ID != 0xFFFF) */
            if (vendor_id == 0xFFFF) {
                continue;
            }
            
            if (pci_device_count_total >= MAX_PCI_DEVICES) {
                serial_puts("[pci] PCI device table full\n");
                return found;
            }
            
            /* Read device configuration */
            pci_device_t *dev = &pci_devices[pci_device_count_total++];
            dev->bus = 0;
            dev->slot = slot;
            dev->function = func;
            dev->pci_device_id = pci_device_count_total - 1;
            
            dev->vendor_id = vendor_id;
            dev->device_id = pci_config_read16(0, slot, func, PCI_REG_DEVICE_ID);
            dev->command = pci_config_read16(0, slot, func, PCI_REG_COMMAND);
            dev->status = pci_config_read16(0, slot, func, PCI_REG_STATUS);
            dev->revision_id = pci_config_read8(0, slot, func, PCI_REG_REVISION_ID);
            dev->class_code = pci_config_read8(0, slot, func, PCI_REG_CLASS_CODE);
            dev->header_type = pci_config_read8(0, slot, func, PCI_REG_HEADER_TYPE);
            dev->irq_line = pci_config_read8(0, slot, func, PCI_REG_IRQ_LINE);
            dev->irq_pin = pci_config_read8(0, slot, func, PCI_REG_IRQ_PIN);
            
            /* Read BARs (Base Address Registers) */
            for (int i = 0; i < 6; i++) {
                uint32_t bar_offset = PCI_REG_BAR0 + (i * 4);
                dev->bars[i] = pci_config_read32(0, slot, func, bar_offset);
            }
            
            serial_printf("[pci] Found device: %04X:%04X (slot=%d, func=%d, class=%02X)\n",
                          vendor_id, dev->device_id, slot, func, dev->class_code);
            
            found++;
        }
    }
    
    serial_printf("[pci] PCI enumeration complete: %d devices found\n", found);
    return found;
}

pci_device_t *pci_find_device(uint16_t vendor_id, uint16_t device_id)
{
    for (int i = 0; i < pci_device_count_total; i++) {
        if (pci_devices[i].vendor_id == vendor_id &&
            pci_devices[i].device_id == device_id) {
            return &pci_devices[i];
        }
    }
    
    return NULL;
}

pci_device_t *pci_find_device_class(uint8_t class_code, uint8_t subclass)
{
    for (int i = 0; i < pci_device_count_total; i++) {
        if (pci_devices[i].class_code == class_code) {
            /* Subclass 0xFF means match any subclass */
            if (subclass == 0xFF) {
                return &pci_devices[i];
            }
            
            uint8_t dev_subclass = pci_config_read8(pci_devices[i].bus, 
                                                     pci_devices[i].slot,
                                                     pci_devices[i].function, 0x0A);
            if (dev_subclass == subclass) {
                return &pci_devices[i];
            }
        }
    }
    
    return NULL;
}

pci_device_t *pci_get_device(int index)
{
    if (index < 0 || index >= pci_device_count_total) {
        return NULL;
    }
    
    return &pci_devices[index];
}

int pci_device_count(void)
{
    return pci_device_count_total;
}

void pci_enable_device(pci_device_t *dev)
{
    if (!dev) return;
    
    /* Enable IO space, memory space, and bus master */
    uint16_t command = dev->command | PCI_CMD_IO_SPACE | PCI_CMD_MEMORY_SPACE | PCI_CMD_BUS_MASTER;
    pci_config_write16(dev->bus, dev->slot, dev->function, PCI_REG_COMMAND, command);
    
    serial_printf("[pci] Enabled device %04X:%04X\n", dev->vendor_id, dev->device_id);
}

void pci_disable_device(pci_device_t *dev)
{
    if (!dev) return;
    
    /* Disable IO space and memory space */
    uint16_t command = dev->command & ~(PCI_CMD_IO_SPACE | PCI_CMD_MEMORY_SPACE);
    pci_config_write16(dev->bus, dev->slot, dev->function, PCI_REG_COMMAND, command);
    
    serial_printf("[pci] Disabled device %04X:%04X\n", dev->vendor_id, dev->device_id);
}

uint32_t pci_get_bar_address(pci_device_t *dev, int bar_num)
{
    if (!dev || bar_num < 0 || bar_num >= 6) {
        return 0;
    }
    
    uint32_t bar = dev->bars[bar_num];
    
    /* IO space BAR */
    if (bar & PCI_BAR_IO) {
        return bar & 0xFFFFFFFC;
    }
    
    /* Memory space BAR */
    return bar & 0xFFFFFFF0;
}

uint32_t pci_get_bar_size(pci_device_t *dev, int bar_num)
{
    if (!dev || bar_num < 0 || bar_num >= 6) {
        return 0;
    }
    
    /* Write all 1s to BAR, read back to get size */
    uint32_t offset = PCI_REG_BAR0 + (bar_num * 4);
    uint32_t original = pci_config_read32(dev->bus, dev->slot, dev->function, offset);
    
    pci_config_write32(dev->bus, dev->slot, dev->function, offset, 0xFFFFFFFF);
    uint32_t size_mask = pci_config_read32(dev->bus, dev->slot, dev->function, offset);
    pci_config_write32(dev->bus, dev->slot, dev->function, offset, original);
    
    if (size_mask == 0 || size_mask == 0xFFFFFFFF) {
        return 0;
    }
    
    /* Calculate size from mask */
    if (original & PCI_BAR_IO) {
        return (~(size_mask & 0xFFFFFFFC)) + 1;
    } else {
        return (~(size_mask & 0xFFFFFFF0)) + 1;
    }
}
