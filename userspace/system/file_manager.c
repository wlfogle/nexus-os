#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <dirent.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <errno.h>
#include <time.h>
#include <pwd.h>
#include <grp.h>
#include <fcntl.h>
#include <linux/limits.h>

#define MAX_FILES 10000
#define BUFFER_SIZE 4096

typedef struct {
    char name[NAME_MAX + 1];
    char path[PATH_MAX];
    off_t size;
    mode_t mode;
    time_t mtime;
    uid_t uid;
    gid_t gid;
    char type;  // 'f' = file, 'd' = directory, 'l' = link
} file_entry_t;

typedef struct {
    file_entry_t files[MAX_FILES];
    int count;
    char current_dir[PATH_MAX];
} file_manager_t;

static file_manager_t fm = {0};

// File type detection based on extension
const char* get_file_type(const char* filename) {
    const char* ext = strrchr(filename, '.');
    if (!ext) return "unknown";
    
    ext++; // Skip the dot
    
    // Code files
    if (strcmp(ext, "c") == 0 || strcmp(ext, "h") == 0) return "C source";
    if (strcmp(ext, "cpp") == 0 || strcmp(ext, "hpp") == 0) return "C++ source";
    if (strcmp(ext, "rs") == 0) return "Rust source";
    if (strcmp(ext, "py") == 0) return "Python script";
    if (strcmp(ext, "js") == 0) return "JavaScript";
    if (strcmp(ext, "go") == 0) return "Go source";
    if (strcmp(ext, "java") == 0) return "Java source";
    
    // Documents
    if (strcmp(ext, "txt") == 0) return "text file";
    if (strcmp(ext, "md") == 0) return "Markdown";
    if (strcmp(ext, "pdf") == 0) return "PDF document";
    if (strcmp(ext, "doc") == 0 || strcmp(ext, "docx") == 0) return "Word document";
    
    // Media
    if (strcmp(ext, "jpg") == 0 || strcmp(ext, "jpeg") == 0 || strcmp(ext, "png") == 0 || strcmp(ext, "gif") == 0) return "image";
    if (strcmp(ext, "mp3") == 0 || strcmp(ext, "wav") == 0 || strcmp(ext, "flac") == 0) return "audio";
    if (strcmp(ext, "mp4") == 0 || strcmp(ext, "avi") == 0 || strcmp(ext, "mkv") == 0) return "video";
    
    // Archives
    if (strcmp(ext, "zip") == 0 || strcmp(ext, "tar") == 0 || strcmp(ext, "gz") == 0) return "archive";
    
    // Config
    if (strcmp(ext, "conf") == 0 || strcmp(ext, "cfg") == 0 || strcmp(ext, "ini") == 0) return "config";
    if (strcmp(ext, "json") == 0) return "JSON config";
    if (strcmp(ext, "yaml") == 0 || strcmp(ext, "yml") == 0) return "YAML config";
    
    return "unknown";
}

// Format file size in human readable format
void format_size(off_t size, char* buffer, size_t buf_size) {
    if (size < 1024) {
        snprintf(buffer, buf_size, "%ld B", size);
    } else if (size < 1024 * 1024) {
        snprintf(buffer, buf_size, "%.1f KB", (double)size / 1024);
    } else if (size < 1024 * 1024 * 1024) {
        snprintf(buffer, buf_size, "%.1f MB", (double)size / (1024 * 1024));
    } else {
        snprintf(buffer, buf_size, "%.1f GB", (double)size / (1024 * 1024 * 1024));
    }
}

// Format permissions string
void format_permissions(mode_t mode, char* buffer) {
    buffer[0] = S_ISDIR(mode) ? 'd' : (S_ISLNK(mode) ? 'l' : '-');
    buffer[1] = (mode & S_IRUSR) ? 'r' : '-';
    buffer[2] = (mode & S_IWUSR) ? 'w' : '-';
    buffer[3] = (mode & S_IXUSR) ? 'x' : '-';
    buffer[4] = (mode & S_IRGRP) ? 'r' : '-';
    buffer[5] = (mode & S_IWGRP) ? 'w' : '-';
    buffer[6] = (mode & S_IXGRP) ? 'x' : '-';
    buffer[7] = (mode & S_IROTH) ? 'r' : '-';
    buffer[8] = (mode & S_IWOTH) ? 'w' : '-';
    buffer[9] = (mode & S_IXOTH) ? 'x' : '-';
    buffer[10] = '\0';
}

// Scan directory and populate file list
int scan_directory(const char* dir_path) {
    DIR *dir;
    struct dirent *entry;
    struct stat file_stat;
    char full_path[PATH_MAX];
    
    dir = opendir(dir_path);
    if (!dir) {
        printf("Error opening directory %s: %s\n", dir_path, strerror(errno));
        return -1;
    }
    
    fm.count = 0;
    strncpy(fm.current_dir, dir_path, PATH_MAX - 1);
    fm.current_dir[PATH_MAX - 1] = '\0';
    
    printf("Scanning directory: %s\n", dir_path);
    
    while ((entry = readdir(dir)) != NULL && fm.count < MAX_FILES) {
        // Skip . and .. but show hidden files
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) {
            continue;
        }
        
        // Build full path
        snprintf(full_path, sizeof(full_path), "%s/%s", dir_path, entry->d_name);
        
        if (stat(full_path, &file_stat) == -1) {
            printf("Warning: Cannot stat %s: %s\n", entry->d_name, strerror(errno));
            continue;
        }
        
        // Store file information
        file_entry_t* file = &fm.files[fm.count];
        strncpy(file->name, entry->d_name, NAME_MAX);
        file->name[NAME_MAX] = '\0';
        strncpy(file->path, full_path, PATH_MAX - 1);
        file->path[PATH_MAX - 1] = '\0';
        file->size = file_stat.st_size;
        file->mode = file_stat.st_mode;
        file->mtime = file_stat.st_mtime;
        file->uid = file_stat.st_uid;
        file->gid = file_stat.st_gid;
        
        if (S_ISDIR(file_stat.st_mode)) {
            file->type = 'd';
        } else if (S_ISLNK(file_stat.st_mode)) {
            file->type = 'l';
        } else {
            file->type = 'f';
        }
        
        fm.count++;
    }
    
    closedir(dir);
    printf("Found %d items\n", fm.count);
    return 0;
}

// List files in current directory
void list_files(int detailed) {
    printf("\nContents of %s:\n", fm.current_dir);
    printf("=====================================\n");
    
    if (detailed) {
        printf("%-10s %3s %-8s %-8s %10s %12s %s\n",
               "Permissions", "Lnk", "Owner", "Group", "Size", "Modified", "Name");
        printf("------------------------------------------------------------------------\n");
    }
    
    for (int i = 0; i < fm.count; i++) {
        file_entry_t* file = &fm.files[i];
        
        if (detailed) {
            char perms[11];
            char size_str[20];
            char time_str[20];
            struct passwd *pw = getpwuid(file->uid);
            struct group *gr = getgrgid(file->gid);
            
            format_permissions(file->mode, perms);
            format_size(file->size, size_str, sizeof(size_str));
            
            struct tm *tm_info = localtime(&file->mtime);
            strftime(time_str, sizeof(time_str), "%b %d %H:%M", tm_info);
            
            printf("%s %3d %-8s %-8s %10s %12s %s%s\n",
                   perms,
                   (int)file->mode & 0777,
                   pw ? pw->pw_name : "?",
                   gr ? gr->gr_name : "?",
                   size_str,
                   time_str,
                   file->name,
                   file->type == 'd' ? "/" : "");
        } else {
            const char* type_str = "";
            const char* color = "";
            const char* reset = "";
            
            if (file->type == 'd') {
                type_str = "/";
                color = "\033[34m"; // Blue for directories
                reset = "\033[0m";
            } else if (file->mode & S_IXUSR) {
                color = "\033[32m"; // Green for executables
                reset = "\033[0m";
            }
            
            printf("%s%s%s%s  ", color, file->name, type_str, reset);
            
            if ((i + 1) % 4 == 0) printf("\n");
        }
    }
    if (!detailed && fm.count % 4 != 0) printf("\n");
    
    printf("\nTotal: %d items\n", fm.count);
}

// Search files by name pattern
void search_files(const char* pattern) {
    printf("\nSearching for files matching '%s':\n", pattern);
    printf("===================================\n");
    
    int matches = 0;
    for (int i = 0; i < fm.count; i++) {
        if (strstr(fm.files[i].name, pattern)) {
            file_entry_t* file = &fm.files[i];
            char size_str[20];
            format_size(file->size, size_str, sizeof(size_str));
            
            printf("  %c %s (%s) - %s\n",
                   file->type == 'd' ? 'D' : (file->type == 'l' ? 'L' : 'F'),
                   file->name,
                   get_file_type(file->name),
                   size_str);
            matches++;
        }
    }
    
    printf("Found %d matches\n", matches);
}

// Read and display file content (first 1KB)
void show_file_content(const char* filename) {
    char filepath[PATH_MAX];
    snprintf(filepath, sizeof(filepath), "%s/%s", fm.current_dir, filename);
    
    int fd = open(filepath, O_RDONLY);
    if (fd == -1) {
        printf("Error opening file %s: %s\n", filename, strerror(errno));
        return;
    }
    
    struct stat st;
    if (fstat(fd, &st) == -1) {
        printf("Error getting file stats: %s\n", strerror(errno));
        close(fd);
        return;
    }
    
    if (S_ISDIR(st.st_mode)) {
        printf("'%s' is a directory\n", filename);
        close(fd);
        return;
    }
    
    printf("\nFile: %s\n", filepath);
    printf("Size: %ld bytes\n", st.st_size);
    printf("Type: %s\n", get_file_type(filename));
    printf("Content preview (first 1KB):\n");
    printf("============================\n");
    
    char buffer[BUFFER_SIZE];
    ssize_t bytes_read = read(fd, buffer, sizeof(buffer) - 1);
    
    if (bytes_read == -1) {
        printf("Error reading file: %s\n", strerror(errno));
    } else if (bytes_read == 0) {
        printf("(empty file)\n");
    } else {
        buffer[bytes_read] = '\0';
        
        // Check if file is binary
        int is_binary = 0;
        for (ssize_t i = 0; i < bytes_read; i++) {
            if (buffer[i] == '\0' || (unsigned char)buffer[i] > 127) {
                is_binary = 1;
                break;
            }
        }
        
        if (is_binary) {
            printf("(binary file - showing hex dump)\n");
            for (ssize_t i = 0; i < bytes_read && i < 256; i += 16) {
                printf("%08lx: ", i);
                for (int j = 0; j < 16 && i + j < bytes_read; j++) {
                    printf("%02x ", (unsigned char)buffer[i + j]);
                }
                printf("\n");
            }
        } else {
            printf("%s", buffer);
            if (st.st_size > bytes_read) {
                printf("\n... (file continues for %ld more bytes)\n", 
                       st.st_size - bytes_read);
            }
        }
    }
    
    close(fd);
}

// Change directory
int change_directory(const char* path) {
    char new_path[PATH_MAX];
    
    if (path[0] == '/') {
        // Absolute path
        strncpy(new_path, path, PATH_MAX - 1);
    } else {
        // Relative path
        snprintf(new_path, sizeof(new_path), "%s/%s", fm.current_dir, path);
    }
    new_path[PATH_MAX - 1] = '\0';
    
    // Resolve . and .. components
    char resolved[PATH_MAX];
    if (realpath(new_path, resolved) == NULL) {
        printf("Error: Cannot access '%s': %s\n", path, strerror(errno));
        return -1;
    }
    
    // Check if it's a directory
    struct stat st;
    if (stat(resolved, &st) == -1) {
        printf("Error: Cannot stat '%s': %s\n", resolved, strerror(errno));
        return -1;
    }
    
    if (!S_ISDIR(st.st_mode)) {
        printf("Error: '%s' is not a directory\n", path);
        return -1;
    }
    
    return scan_directory(resolved);
}

// File statistics
void show_stats() {
    int files = 0, dirs = 0, links = 0;
    off_t total_size = 0;
    
    for (int i = 0; i < fm.count; i++) {
        switch (fm.files[i].type) {
            case 'f': files++; total_size += fm.files[i].size; break;
            case 'd': dirs++; break;
            case 'l': links++; break;
        }
    }
    
    char size_str[20];
    format_size(total_size, size_str, sizeof(size_str));
    
    printf("\nDirectory Statistics:\n");
    printf("====================\n");
    printf("Files:       %d\n", files);
    printf("Directories: %d\n", dirs);
    printf("Links:       %d\n", links);
    printf("Total size:  %s\n", size_str);
}

// Help function
void show_help() {
    printf("\nNexusOS File Manager - Available Commands:\n");
    printf("==========================================\n");
    printf("ls, list         - List files in current directory\n");
    printf("ll, detail       - List files with detailed information\n");
    printf("cd <path>        - Change to directory\n");
    printf("cat <file>       - Show file content\n");
    printf("search <pattern> - Search for files by name\n");
    printf("stats            - Show directory statistics\n");
    printf("pwd              - Show current directory\n");
    printf("refresh          - Rescan current directory\n");
    printf("help             - Show this help\n");
    printf("quit, exit       - Exit file manager\n");
    printf("\nExamples:\n");
    printf("  cd /home\n");
    printf("  cat README.md\n");
    printf("  search .txt\n");
}

// Main interactive loop
int main(int argc, char* argv[]) {
    char* start_dir = (argc > 1) ? argv[1] : ".";
    char input[256];
    char command[64], argument[256];
    
    printf("NexusOS File Manager v1.0\n");
    printf("=========================\n");
    
    // Initial directory scan
    if (scan_directory(start_dir) != 0) {
        return 1;
    }
    
    list_files(0);
    
    // Main command loop
    while (1) {
        printf("\n[%s]$ ", fm.current_dir);
        fflush(stdout);
        
        if (!fgets(input, sizeof(input), stdin)) {
            break;
        }
        
        // Remove newline
        input[strcspn(input, "\n")] = 0;
        
        // Skip empty input
        if (strlen(input) == 0) {
            continue;
        }
        
        // Parse command and argument
        if (sscanf(input, "%63s %255s", command, argument) < 1) {
            continue;
        }
        
        // Execute commands
        if (strcmp(command, "quit") == 0 || strcmp(command, "exit") == 0) {
            break;
        } else if (strcmp(command, "ls") == 0 || strcmp(command, "list") == 0) {
            list_files(0);
        } else if (strcmp(command, "ll") == 0 || strcmp(command, "detail") == 0) {
            list_files(1);
        } else if (strcmp(command, "cd") == 0) {
            if (sscanf(input, "%*s %255s", argument) == 1) {
                change_directory(argument);
                list_files(0);
            } else {
                printf("Usage: cd <directory>\n");
            }
        } else if (strcmp(command, "cat") == 0) {
            if (sscanf(input, "%*s %255s", argument) == 1) {
                show_file_content(argument);
            } else {
                printf("Usage: cat <filename>\n");
            }
        } else if (strcmp(command, "search") == 0) {
            if (sscanf(input, "%*s %255s", argument) == 1) {
                search_files(argument);
            } else {
                printf("Usage: search <pattern>\n");
            }
        } else if (strcmp(command, "stats") == 0) {
            show_stats();
        } else if (strcmp(command, "pwd") == 0) {
            printf("%s\n", fm.current_dir);
        } else if (strcmp(command, "refresh") == 0) {
            scan_directory(fm.current_dir);
            list_files(0);
        } else if (strcmp(command, "help") == 0) {
            show_help();
        } else {
            printf("Unknown command: %s\n", command);
            printf("Type 'help' for available commands\n");
        }
    }
    
    printf("\nGoodbye!\n");
    return 0;
}