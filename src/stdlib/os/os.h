#ifndef PEEL_OS_H
#define PEEL_OS_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>

char* peel_os_get_env(const char* name);
int peel_os_set_env(const char* name, const char* value);
char* peel_os_cwd();
char* peel_os_platform();
char* peel_os_arch();
long long peel_os_uptime();
char* peel_os_hostname();
double peel_os_cpu_usage();
unsigned long long peel_os_total_memory();
unsigned long long peel_os_free_memory();
void peel_os_free(void* ptr);

#ifdef __cplusplus
}
#endif

#endif // PEEL_OS_H
