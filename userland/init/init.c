#include "../lib/libc.h"

int main(void)
{
    printf("===== NexusOS Init Process =====\n");
    printf("Init: system initialized\n");
    while (1) {
        sleep(1000);
    }
    return 0;
}
