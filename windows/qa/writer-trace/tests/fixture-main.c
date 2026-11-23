#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

extern int32_t NtWriteFile(void *, void *, void *, void *, void *,
                          const void *, uint32_t, void *, void *);
extern int32_t NtDeviceIoControlFile(void *, void *, void *, void *, void *,
                                    uint32_t, const void *, uint32_t,
                                    void *, uint32_t);
extern int HidD_SetOutputReport(void *, const void *, uint32_t);
extern void GetSystemTimePreciseAsFileTime(uint64_t *);

__attribute__((noinline))
static int run_batch(void) {
    unsigned char motor[11] = {0x31, 1, 0xe0, 0xfe, 0, 0, 1, 0xe0, 0xfe, 0, 0};
    unsigned char padded[95] = {0x31, 1, 0x20, 1, 0, 0, 1, 0x20, 1, 0x86, 0x29};
    unsigned char ring[41] = {0x32, 0x12, 0x34};
    unsigned char embedded[14] = {0xaa, 0xbb, 0xcc};
    unsigned char irrelevant[5] = {0x31, 1, 0, 0, 0};
    memcpy(embedded + 3, motor, sizeof motor);
    int failures = 0;
    failures += NtWriteFile((void *)0x100, 0, 0, 0, 0, motor, sizeof motor, 0, 0) != 0;
    failures += NtWriteFile((void *)0x101, 0, 0, 0, 0, padded, sizeof padded, 0, 0) != 0;
    failures += NtWriteFile((void *)0x102, 0, 0, 0, 0, irrelevant, sizeof irrelevant, 0, 0) != 0;
    failures += NtDeviceIoControlFile((void *)0x200, 0, 0, 0, 0, 0xdead002,
                                    embedded, sizeof embedded, 0, 0) != 0;
    failures += NtDeviceIoControlFile((void *)0x201, 0, 0, 0, 0, 0xbeef002,
                                    0, 0, ring, sizeof ring) != 0;
    failures += HidD_SetOutputReport((void *)0x300, ring, sizeof ring) != 1;
    failures += NtWriteFile((void *)0x103, 0, 0, 0, 0, motor, sizeof motor, 0, 0)
        != (int32_t)0xc0000008;
    failures += NtWriteFile((void *)0x104, 0, 0, 0, 0, motor, sizeof motor, 0, 0)
        != 0x103;
    failures += motor[0] != 0x31 || padded[9] != 0x86 || ring[1] != 0x12;
    return failures;
}

int main(int argc, char **argv) {
    if (argc != 1) return 2;
    uint64_t time;
    GetSystemTimePreciseAsFileTime(&time);
    if (!time) return 6;
    size_t size = strlen(argv[0]) + 16;
    char *pipe_path = malloc(size);
    char *reply_path = malloc(size);
    if (!pipe_path || !reply_path) return 3;
    snprintf(pipe_path, size, "%s.pipe", argv[0]);
    snprintf(reply_path, size, "%s.reply", argv[0]);
    char *pid_path = malloc(size);
    if (!pid_path) return 7;
    snprintf(pid_path, size, "%s.pid", argv[0]);
    FILE *pid_file = fopen(pid_path, "w");
    if (!pid_file) return 8;
    fprintf(pid_file, "%d", getpid());
    fclose(pid_file);
    free(pid_path);
    FILE *pipe = fopen(pipe_path, "r");
    if (!pipe) return 4;
    char line[80];
    while (fgets(line, sizeof line, pipe)) {
        if (strncmp(line, "BATCH ", 6) != 0) break;
        int batch = atoi(line + 6);
        int failures = run_batch();
        FILE *reply = fopen(reply_path, "a");
        if (!reply) return 5;
        fprintf(reply, "{\"batch\":%d,\"failures\":%d}\n", batch, failures);
        fclose(reply);
    }
    fclose(pipe);
    free(reply_path);
    free(pipe_path);
    return 0;
}
