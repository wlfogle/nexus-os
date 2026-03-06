#ifndef KERNEL_KSHELL_H
#define KERNEL_KSHELL_H

/* Dynamic command handler: receives (argc, argv) */
typedef void (*kshell_cmd_fn)(int argc, char *argv[]);

/* Register a dynamic command.  Returns 0 on success, -1 if table full. */
int kshell_register_command(const char *name, const char *desc, kshell_cmd_fn fn);

/* Unregister a dynamic command by name.  Returns 0 on success. */
int kshell_unregister_command(const char *name);

/* Start the interactive kernel shell (never returns) */
void kshell_run(void);

#endif /* KERNEL_KSHELL_H */
