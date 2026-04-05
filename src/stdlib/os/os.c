#define _GNU_SOURCE
#include "os.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

#ifdef _WIN32
    #include <windows.h>
#else
    #include <unistd.h>
    #include <sys/utsname.h>
    #include <pthread.h>
    #include <sys/types.h>
    #ifdef __APPLE__
        #include <sys/sysctl.h>
        #include <mach/mach.h>
    #else
        #include <sys/sysinfo.h>
    #endif
#endif

static char* safe_strdup(const char* s) {
    if (s == NULL) return NULL;
    char* res = malloc(strlen(s) + 1);
    if (res) strcpy(res, s);
    return res;
}

char* peel_os_get_env(const char* name) {
#ifdef _WIN32
    DWORD size = GetEnvironmentVariable(name, NULL, 0);
    if (size == 0) return NULL;
    char* buffer = (char*)malloc(size);
    if (buffer) {
        if (GetEnvironmentVariable(name, buffer, size) == 0) {
            free(buffer);
            return NULL;
        }
    }
    return buffer; 
#else
    char* val = getenv(name);
    return val ? safe_strdup(val) : NULL;
#endif
}

char* peel_os_cwd() {
#ifdef _WIN32
    DWORD size = GetCurrentDirectory(0, NULL);
    if (size == 0) return NULL;
    char* buffer = (char*)malloc(size);
    if (buffer) {
        if (GetCurrentDirectory(size, buffer) == 0) {
            free(buffer);
            return NULL;
        }
    }
    return buffer;
#else
    char* buffer = getcwd(NULL, 0);
    if (buffer) return buffer;
    size_t size = 1024;
    while (1) {
        buffer = malloc(size);
        if (getcwd(buffer, size)) return buffer;
        free(buffer);
        if (errno != ERANGE) return NULL;
        size *= 2;
    }
#endif
}

#ifdef _WIN32
    static FILETIME p_idle, p_kernel, p_user;
    static int first_cpu_call = 1;
    static SRWLOCK cpu_lock = SRWLOCK_INIT;
    static unsigned long long ft_to_ull(FILETIME ft) {
        return ((unsigned long long)ft.dwHighDateTime << 32) | ft.dwLowDateTime;
    }
#else
    static pthread_mutex_t cpu_lock = PTHREAD_MUTEX_INITIALIZER;
    static unsigned long long p_total, p_idle;
    static int first_cpu_call = 1;
#endif

double peel_os_cpu_usage() {
#ifdef _WIN32
    AcquireSRWLockExclusive(&cpu_lock);
    FILETIME idle, kernel, user;
    if (!GetSystemTimes(&idle, &kernel, &user)) {
        ReleaseSRWLockExclusive(&cpu_lock); return 0.0;
    }
    if (first_cpu_call) {
        p_idle = idle; p_kernel = kernel; p_user = user;
        first_cpu_call = 0; ReleaseSRWLockExclusive(&cpu_lock); return 0.0;
    }
    unsigned long long d_idle = ft_to_ull(idle) - ft_to_ull(p_idle);
    unsigned long long d_kernel = ft_to_ull(kernel) - ft_to_ull(p_kernel);
    unsigned long long d_user = ft_to_ull(user) - ft_to_ull(p_user);
    p_idle = idle; p_kernel = kernel; p_user = user;
    ReleaseSRWLockExclusive(&cpu_lock);

    unsigned long long total = d_kernel + d_user;
    return (total > 0) ? (double)(total - d_idle) / total * 100.0 : 0.0;

#elif defined(__APPLE__)
    pthread_mutex_lock(&cpu_lock);
    host_cpu_load_info_data_t load;
    mach_msg_type_number_t count = HOST_CPU_LOAD_INFO_COUNT;
    if (host_statistics(mach_host_self(), HOST_CPU_LOAD_INFO, (host_info_t)&load, &count) != KERN_SUCCESS) {
        pthread_mutex_unlock(&cpu_lock); return 0.0;
    }
    unsigned long long user = load.cpu_ticks[CPU_STATE_USER], sys = load.cpu_ticks[CPU_STATE_SYSTEM],
                       idle = load.cpu_ticks[CPU_STATE_IDLE], nice = load.cpu_ticks[CPU_STATE_NICE];
    unsigned long long total = user + sys + idle + nice;
    if (first_cpu_call) {
        p_total = total; p_idle = idle;
        first_cpu_call = 0; pthread_mutex_unlock(&cpu_lock); return 0.0;
    }
    unsigned long long d_total = total - p_total;
    unsigned long long d_idle = idle - p_idle;
    p_total = total; p_idle = idle;
    pthread_mutex_unlock(&cpu_lock);
    return (d_total > 0) ? (double)(d_total - d_idle) / d_total * 100.0 : 0.0;

#else // Linux
    pthread_mutex_lock(&cpu_lock);
    FILE* fp = fopen("/proc/stat", "r");
    if (!fp) { pthread_mutex_unlock(&cpu_lock); return 0.0; }
    unsigned long long u, n, s, i, iw, irq, sirq;
    // Reading 7 columns to include iowait and interrupts for accuracy
    if (fscanf(fp, "cpu %llu %llu %llu %llu %llu %llu %llu", &u, &n, &s, &i, &iw, &irq, &sirq) < 7) {
        fclose(fp); pthread_mutex_unlock(&cpu_lock); return 0.0;
    }
    fclose(fp);
    unsigned long long total = u + n + s + i + iw + irq + sirq;
    unsigned long long idle = i + iw; 
    if (first_cpu_call) {
        p_total = total; p_idle = idle;
        first_cpu_call = 0; pthread_mutex_unlock(&cpu_lock); return 0.0;
    }
    unsigned long long d_total = total - p_total;
    unsigned long long d_idle = idle - p_idle;
    p_total = total; p_idle = idle;
    pthread_mutex_unlock(&cpu_lock);
    return (d_total > 0) ? (double)(d_total - d_idle) / d_total * 100.0 : 0.0;
#endif
}

char* peel_os_platform() {
#ifdef _WIN32
    return safe_strdup("win32");
#elif __APPLE__
    return safe_strdup("darwin");
#elif __linux__
    return safe_strdup("linux");
#else
    return safe_strdup("unknown");
#endif
}

char* peel_os_arch() {
#ifdef _WIN32
    SYSTEM_INFO si;
    GetSystemInfo(&si);
    switch(si.wProcessorArchitecture) {
        case PROCESSOR_ARCHITECTURE_AMD64: return safe_strdup("x64");
        case PROCESSOR_ARCHITECTURE_INTEL: return safe_strdup("x86");
        case PROCESSOR_ARCHITECTURE_ARM64: return safe_strdup("arm64");
        default: return safe_strdup("unknown");
    }
#else
    struct utsname buffer;
    if (uname(&buffer) == 0) {
        if (strcmp(buffer.machine, "x86_64") == 0) return safe_strdup("x64");
        if (strcmp(buffer.machine, "aarch64") == 0 || strcmp(buffer.machine, "arm64") == 0) 
            return safe_strdup("arm64");
        return safe_strdup(buffer.machine);
    }
    return safe_strdup("unknown");
#endif
}

unsigned long long peel_os_total_memory() {
#ifdef _WIN32
    MEMORYSTATUSEX status; status.dwLength = sizeof(status);
    return GlobalMemoryStatusEx(&status) ? status.ullTotalPhys : 0;
#elif defined(__APPLE__)
    int64_t mem; size_t len = sizeof(mem);
    return (sysctlbyname("hw.memsize", &mem, &len, NULL, 0) == 0) ? (unsigned long long)mem : 0;
#else
    struct sysinfo info;
    return (sysinfo(&info) == 0) ? (unsigned long long)info.totalram * info.mem_unit : 0;
#endif
}

unsigned long long peel_os_free_memory() {
#ifdef _WIN32
    MEMORYSTATUSEX status; status.dwLength = sizeof(status);
    return GlobalMemoryStatusEx(&status) ? status.ullAvailPhys : 0;
#elif defined(__APPLE__)
    mach_msg_type_number_t count = HOST_VM_INFO64_COUNT;
    vm_statistics64_data_t vm_stats;
    if (host_statistics64(mach_host_self(), HOST_VM_INFO64, (host_info64_t)&vm_stats, &count) == KERN_SUCCESS) {
        return (unsigned long long)vm_stats.free_count * sysconf(_SC_PAGESIZE);
    }
    return 0;
#else
    struct sysinfo info;
    return (sysinfo(&info) == 0) ? (unsigned long long)info.freeram * info.mem_unit : 0;
#endif
}

long long peel_os_uptime() {
#ifdef _WIN32
    return GetTickCount64() / 1000;
#else
    struct sysinfo info;
    if (sysinfo(&info) == 0) return (long long)info.uptime;
    return -1;
#endif
}

char* peel_os_hostname() {
#ifdef _WIN32
    char buffer[MAX_COMPUTERNAME_LENGTH + 1];
    DWORD size = sizeof(buffer);
    if (GetComputerName(buffer, &size)) return safe_strdup(buffer);
#else
    char buffer[256];
    if (gethostname(buffer, sizeof(buffer)) == 0) return safe_strdup(buffer);
#endif
    return safe_strdup("localhost");
}

void peel_os_free(void* ptr) { if (ptr) free(ptr); }
