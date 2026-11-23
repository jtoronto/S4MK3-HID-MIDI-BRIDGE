#include <stdint.h>

// These exports are harmless ABI fixtures. No Windows or device API is called.
__attribute__((visibility("default"), noinline))
int32_t NtWriteFile(void *handle, void *event, void *apc, void *context,
                   void *status, const void *buffer, uint32_t length,
                   void *offset, void *key) {
    (void)event; (void)apc; (void)context; (void)status;
    (void)offset; (void)key;
    if (length && !buffer) return (int32_t)0xc000000d;
    if ((uintptr_t)handle == 0x103) return (int32_t)0xc0000008;
    if ((uintptr_t)handle == 0x104) return 0x103;
    return 0;
}

__attribute__((visibility("default"), noinline))
int32_t NtDeviceIoControlFile(void *handle, void *event, void *apc,
                            void *context, void *status, uint32_t ioctl,
                            const void *input, uint32_t input_length,
                            void *output, uint32_t output_length) {
    (void)handle; (void)event; (void)apc; (void)context; (void)status;
    (void)ioctl; (void)input; (void)input_length;
    (void)output; (void)output_length;
    return 0;
}
