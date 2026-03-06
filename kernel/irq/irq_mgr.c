#include "../../include/kernel/irq_mgr.h"
#include "../../include/kernel/pic.h"
#include "../../include/kernel/serial.h"
#include <string.h>

#define MAX_IRQS 16

typedef struct {
    irq_handler_entry_t handlers[MAX_HANDLERS_PER_IRQ];
    irq_stats_t stats;
    int enabled;
} irq_line_t;

static irq_line_t irq_lines[MAX_IRQS];

void irq_mgr_init(void)
{
    memset(irq_lines, 0, sizeof(irq_lines));
    
    for (int i = 0; i < MAX_IRQS; i++) {
        irq_lines[i].stats.irq_num = i;
        irq_lines[i].enabled = 1;
    }
    
    serial_puts("[irq_mgr] Interrupt manager initialized\n");
}

int irq_register_handler(uint32_t irq, irq_handler_t handler, void *dev_data,
                         uint32_t device_id, irq_priority_t priority)
{
    if (irq >= MAX_IRQS || !handler || priority < 1 || priority > 4) {
        return -1;
    }
    
    irq_line_t *line = &irq_lines[irq];
    
    /* Find free handler slot */
    int free_slot = -1;
    for (int i = 0; i < MAX_HANDLERS_PER_IRQ; i++) {
        if (!line->handlers[i].active) {
            free_slot = i;
            break;
        }
    }
    
    if (free_slot < 0) {
        serial_printf("[irq_mgr] No free handler slots for IRQ %d\n", irq);
        return -1;
    }
    
    /* Register handler */
    irq_handler_entry_t *entry = &line->handlers[free_slot];
    entry->handler = handler;
    entry->dev_data = dev_data;
    entry->device_id = device_id;
    entry->priority = priority;
    entry->active = 1;
    
    line->stats.handler_count++;
    
    serial_printf("[irq_mgr] Registered handler for IRQ %d (device %d, priority %d)\n",
                  irq, device_id, priority);
    
    /* Enable IRQ at PIC */
    pic_enable_irq((uint8_t)irq);
    
    return 0;
}

int irq_unregister_handler(uint32_t irq, uint32_t device_id)
{
    if (irq >= MAX_IRQS) return -1;
    
    irq_line_t *line = &irq_lines[irq];
    
    for (int i = 0; i < MAX_HANDLERS_PER_IRQ; i++) {
        if (line->handlers[i].active && line->handlers[i].device_id == device_id) {
            line->handlers[i].active = 0;
            line->stats.handler_count--;
            
            serial_printf("[irq_mgr] Unregistered handler for IRQ %d (device %d)\n",
                          irq, device_id);
            
            /* Disable IRQ if no handlers left */
            if (line->stats.handler_count == 0) {
                pic_disable_irq((uint8_t)irq);
            }
            
            return 0;
        }
    }
    
    return -1;
}

/* Sort handlers by priority (high to low) */
static int __attribute__((unused)) compare_priority(const void *a, const void *b)
{
    const irq_handler_entry_t *ha = (const irq_handler_entry_t *)a;
    const irq_handler_entry_t *hb = (const irq_handler_entry_t *)b;
    
    if (!ha->active) return 1;
    if (!hb->active) return -1;
    
    return hb->priority - ha->priority;
}

int irq_dispatch_handlers(uint32_t irq)
{
    if (irq >= MAX_IRQS) return -1;
    
    irq_line_t *line = &irq_lines[irq];
    line->stats.total_interrupts++;
    
    /* Sort handlers by priority */
    irq_handler_entry_t temp[MAX_HANDLERS_PER_IRQ];
    memcpy(temp, line->handlers, sizeof(temp));
    
    int handled = 0;
    for (int i = 0; i < MAX_HANDLERS_PER_IRQ; i++) {
        if (!temp[i].active) continue;
        
        if (temp[i].handler) {
            int ret = temp[i].handler(irq, temp[i].dev_data);
            if (ret == 0) {
                handled = 1;
                break;  /* Handler consumed interrupt */
            }
        }
    }
    
    if (handled) {
        line->stats.handled_count++;
    } else {
        line->stats.missed_count++;
    }
    
    return handled ? 0 : -1;
}

irq_handler_t irq_get_handler(uint32_t irq, uint32_t device_id)
{
    if (irq >= MAX_IRQS) return NULL;
    
    irq_line_t *line = &irq_lines[irq];
    
    for (int i = 0; i < MAX_HANDLERS_PER_IRQ; i++) {
        if (line->handlers[i].active && line->handlers[i].device_id == device_id) {
            return line->handlers[i].handler;
        }
    }
    
    return NULL;
}

int irq_enable(uint32_t irq)
{
    if (irq >= MAX_IRQS) return -1;
    
    irq_lines[irq].enabled = 1;
    pic_enable_irq((uint8_t)irq);
    
    return 0;
}

int irq_disable(uint32_t irq)
{
    if (irq >= MAX_IRQS) return -1;
    
    irq_lines[irq].enabled = 0;
    pic_disable_irq((uint8_t)irq);
    
    return 0;
}

irq_stats_t *irq_get_stats(uint32_t irq)
{
    if (irq >= MAX_IRQS) return NULL;
    
    return &irq_lines[irq].stats;
}

int irq_get_handler_count(uint32_t irq)
{
    if (irq >= MAX_IRQS) return -1;
    
    return irq_lines[irq].stats.handler_count;
}
