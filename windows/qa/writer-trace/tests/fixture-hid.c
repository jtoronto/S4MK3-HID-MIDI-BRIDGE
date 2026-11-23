#include <stdint.h>

extern int32_t NtDeviceIoControlFile(void *, void *, void *, void *, void *,
                                   uint32_t, const void *, uint32_t,
                                   void *, uint32_t);

__attribute__((visibility("default"), noinline))
int HidD_SetOutputReport(void *handle, const void *report, uint32_t size) {
    NtDeviceIoControlFile(handle, 0, 0, 0, 0, 0xb0191, report, size, 0, 0);
    return 1;
}
