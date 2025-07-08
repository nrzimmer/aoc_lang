#include <unistd.h>
#include <stdlib.h>
#include <string.h>

char* read_until_eof(int fd) {
    size_t capacity = 4096;
    size_t used = 0;
    char *buffer = malloc(capacity + 1);  // +1 for null terminator
    ssize_t bytes_read;

    while ((bytes_read = read(fd, buffer + used, capacity - used)) > 0) {
        used += bytes_read;
        if (used == capacity) {
            capacity *= 2;
            buffer = realloc(buffer, capacity + 1);  // +1 for null terminator
        }
    }

    buffer = realloc(buffer, used + 1);  // Shrink to actual size + null terminator
    buffer[used] = '\0';
    return buffer;
}
