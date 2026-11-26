#include <stdint.h>
#include <time.h>

__attribute__((visibility("default"), noinline))
void GetSystemTimePreciseAsFileTime(uint64_t *value) {
    struct timespec now;
    clock_gettime(CLOCK_REALTIME, &now);
    *value = UINT64_C(116444736000000000) +
        (uint64_t)now.tv_sec * UINT64_C(10000000) +
        (uint64_t)now.tv_nsec / 100;
}
