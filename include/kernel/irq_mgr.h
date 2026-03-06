#ifndef KERNEL_IRQ_MGR_H
#define KERNEL_IRQ_MGR_H

#include "../libc/stdint.h"

/* Maximum handlers per IRQ */
#define MAX_HANDLERS_PER_IRQ 4

/* IRQ handler function type */
typedef int (*irq_handler_t)(uint32_t irq, void *dev_data);

/* Interrupt priority levels */
typedef enum {
    IRQ_PRIORITY_LOW = 1,
    IRQ_PRIORITY_NORMAL = 2,
    IRQ_PRIORITY_HIGH = 3,
    IRQ_PRIORITY_CRITICAL = 4
} irq_priority_t;

/* IRQ handler entry */
typedef struct {
    irq_handler_t handler;
    void *dev_data;
    uint32_t device_id;
    irq_priority_t priority;
    int active;
} irq_handler_entry_t;

/* IRQ statistics */
typedef struct {
    uint32_t irq_num;
    uint32_t total_interrupts;
    uint32_t handled_count;
    uint32_t missed_count;
    uint32_t handler_count;
} irq_stats_t;

/* IRQ management API */
void irq_mgr_init(void);

/* Handler registration/unregistration */
int irq_register_handler(uint32_t irq, irq_handler_t handler, void *dev_data, 
                         uint32_t device_id, irq_priority_t priority);
int irq_unregister_handler(uint32_t irq, uint32_t device_id);

/* Handler lookup and dispatch */
int irq_dispatch_handlers(uint32_t irq);
irq_handler_t irq_get_handler(uint32_t irq, uint32_t device_id);

/* IRQ enable/disable */
int irq_enable(uint32_t irq);
int irq_disable(uint32_t irq);

/* Statistics */
irq_stats_t *irq_get_stats(uint32_t irq);
int irq_get_handler_count(uint32_t irq);

#endif /* KERNEL_IRQ_MGR_H */
