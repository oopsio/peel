#include "os.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <windows.h>
#include <sysinfoapi.h>
#else
#include <unistd.h>
#include <sys/utsname.h>
#include <sys/sysinfo.h>
#endif

char* peel_os_get_env(const char* name) {
#ifdef _WIN32
    DWORD size = GetEnvironmentVariable(name, NULL, 0);
    if (size == 0) return NULL;
    char* buffer = (char*)malloc(size);
    if (!buffer) return NULL;
    GetEnvironmentVariable(name, buffer, size);
    return buffer; // Rust will free this since peel_os_free uses free()
#else
    char* val = getenv(name);
    if (val == NULL) return NULL;
    return strdup(val);
#endif
}

int peel_os_set_env(const char* name, const char* value) {
#ifdef _WIN32
    return SetEnvironmentVariable(name, value) ? 0 : -1;
#else
    return setenv(name, value, 1);
#endif
}

char* peel_os_cwd() {
#ifdef _WIN32
    DWORD size = GetCurrentDirectory(0, NULL);
    if (size == 0) return NULL;
    char* buffer = (char*)malloc(size);
    if (!buffer) return NULL;
    GetCurrentDirectory(size, buffer);
    return buffer;
#else
    char buffer[1024]; // Standard limit
    if (getcwd(buffer, sizeof(buffer)) != NULL) {
        return strdup(buffer);
    }
    return NULL;
#endif
}

char* peel_os_platform() {
#ifdef _WIN32
    return strdup("win32");
#elif __APPLE__
    return strdup("darwin");
#elif __linux__
    return strdup("linux");
#else
    return strdup("unknown");
#endif
}

char* peel_os_arch() {
#ifdef _WIN32
    SYSTEM_INFO si;
    GetSystemInfo(&si);
    if (si.wProcessorArchitecture == PROCESSOR_ARCHITECTURE_AMD64) return strdup("x64");
    if (si.wProcessorArchitecture == PROCESSOR_ARCHITECTURE_INTEL) return strdup("x86");
    if (si.wProcessorArchitecture == PROCESSOR_ARCHITECTURE_ARM64) return strdup("arm64");
    return strdup("unknown");
#else
    struct utsname buffer;
    if (uname(&buffer) == 0) {
        return strdup(buffer.machine);
    }
    return strdup("unknown");
#endif
}

long long peel_os_uptime() {
#ifdef _WIN32
    return GetTickCount64() / 1000;
#else
    struct sysinfo info;
    if (sysinfo(&info) == 0) {
        return info.uptime;
    }
    return -1;
#endif
}

char* peel_os_hostname() {
#ifdef _WIN32
    DWORD size = 0;
    GetComputerName(NULL, &size); // Returns required size including NULL
    if (size == 0) size = 256;
    char* buffer = (char*)malloc(size);
    if (!buffer) return strdup("localhost");
    if (GetComputerName(buffer, &size)) {
        return buffer;
    }
    free(buffer);
#else
    char buffer[256];
    if (gethostname(buffer, sizeof(buffer)) == 0) {
        return strdup(buffer);
    }
#endif
    return strdup("localhost");
}

#ifdef _WIN32
static FILETIME prev_idle_time, prev_kernel_time, prev_user_time;
static int first_cpu_call = 1;
static SRWLOCK cpu_lock = SRWLOCK_INIT;

static unsigned long long filetime_to_ull(FILETIME ft) {
    return ((unsigned long long)ft.dwHighDateTime << 32) | ft.dwLowDateTime;
}
#endif

double peel_os_cpu_usage() {
#ifdef _WIN32
    AcquireSRWLockExclusive(&cpu_lock);

    FILETIME idle_time, kernel_time, user_time;
    if (!GetSystemTimes(&idle_time, &kernel_time, &user_time)) {
        ReleaseSRWLockExclusive(&cpu_lock);
        return 0.0;
    }

    if (first_cpu_call) {
        prev_idle_time = idle_time;
        prev_kernel_time = kernel_time;
        prev_user_time = user_time;
        first_cpu_call = 0;
        ReleaseSRWLockExclusive(&cpu_lock);
        return 0.0;
    }

    unsigned long long idle = filetime_to_ull(idle_time) - filetime_to_ull(prev_idle_time);
    unsigned long long kernel = filetime_to_ull(kernel_time) - filetime_to_ull(prev_kernel_time);
    unsigned long long user = filetime_to_ull(user_time) - filetime_to_ull(prev_user_time);
    unsigned long long kernel_plus_user = kernel + user;

    prev_idle_time = idle_time;
    prev_kernel_time = kernel_time;
    prev_user_time = user_time;

    double result = 0.0;
    if (kernel_plus_user != 0) {
        result = (double)(kernel_plus_user - idle) / (double)kernel_plus_user * 100.0;
    }

    ReleaseSRWLockExclusive(&cpu_lock);
    return result;
#else
    // For Linux, we could read /proc/stat
    return 0.0; 
#endif
}

unsigned long long peel_os_total_memory() {
#ifdef _WIN32
    MEMORYSTATUSEX status;
    status.dwLength = sizeof(status);
    if (GlobalMemoryStatusEx(&status)) {
        return status.ullTotalPhys;
    }
#else
    struct sysinfo info;
    if (sysinfo(&info) == 0) {
        return (unsigned long long)info.totalram * info.mem_unit;
    }
#endif
    return 0;
}

unsigned long long peel_os_free_memory() {
#ifdef _WIN32
    MEMORYSTATUSEX status;
    status.dwLength = sizeof(status);
    if (GlobalMemoryStatusEx(&status)) {
        return status.ullAvailPhys;
    }
#else
    struct sysinfo info;
    if (sysinfo(&info) == 0) {
        return (unsigned long long)info.freeram * info.mem_unit;
    }
#endif
    return 0;
}

void peel_os_free(void* ptr) {
    free(ptr);
}
