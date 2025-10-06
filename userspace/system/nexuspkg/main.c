#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <errno.h>
#include <time.h>
#include <stdarg.h>
#include <curl/curl.h>
#include <json-c/json.h>
#include <sqlite3.h>
#include <archive.h>
#include <archive_entry.h>
#include <openssl/sha.h>

#define NEXUSPKG_VERSION "1.0.0"
#define NEXUSPKG_DB_PATH "/var/lib/nexuspkg/packages.db"
#define NEXUSPKG_CACHE_PATH "/var/cache/nexuspkg"
#define NEXUSPKG_CONFIG_PATH "/etc/nexuspkg/nexuspkg.conf"
#define NEXUSPKG_REPO_URL "https://packages.nexusos.org"
#define NEXUSPKG_AI_MODELS_REPO "https://models.nexusos.org"

#define MAX_PACKAGE_NAME 256
#define MAX_VERSION 64
#define MAX_DESCRIPTION 1024
#define MAX_URL 512
#define MAX_DEPENDENCIES 32

typedef enum {
    PKG_TYPE_SOFTWARE = 0,
    PKG_TYPE_AI_MODEL,
    PKG_TYPE_DATASET,
    PKG_TYPE_LIBRARY,
    PKG_TYPE_KERNEL_MODULE,
    PKG_TYPE_DESKTOP_APP
} package_type_t;

typedef enum {
    PKG_FORMAT_NATIVE = 0,    // NexusOS native .npkg format
    PKG_FORMAT_DEB,           // Debian/Ubuntu .deb packages
    PKG_FORMAT_RPM,           // RedHat/SUSE .rpm packages
    PKG_FORMAT_ZST,           // Arch Linux .pkg.tar.zst packages
    PKG_FORMAT_APPIMAGE,      // Universal AppImage packages
    PKG_FORMAT_FLATPAK,       // Flatpak sandboxed packages
    PKG_FORMAT_SNAP,          // Ubuntu Snap packages
    PKG_FORMAT_TAR_XZ,        // Generic tar.xz archives
    PKG_FORMAT_TAR_GZ,        // Generic tar.gz archives
    PKG_FORMAT_ZIP,           // ZIP archives
    PKG_FORMAT_BINARY,        // Raw binary executables
    PKG_FORMAT_PYTHON_WHEEL,  // Python wheel packages
    PKG_FORMAT_NPM,           // Node.js npm packages
    PKG_FORMAT_CARGO,         // Rust cargo packages
    PKG_FORMAT_DOCKER,        // Docker container images
    PKG_FORMAT_OCI            // OCI container images
} package_format_t;

typedef struct {
    char name[MAX_PACKAGE_NAME];
    char version[MAX_VERSION];
    char description[MAX_DESCRIPTION];
    char download_url[MAX_URL];
    char checksum[65]; // SHA-256
    uint64_t size;
    package_type_t type;
    package_format_t format;
    char source_repo[MAX_URL];    // Source repository (AUR, Ubuntu, etc.)
    char architecture[32];        // Target architecture
    char dependencies[MAX_DEPENDENCIES][MAX_PACKAGE_NAME];
    int dependency_count;
    int installed;
    uint64_t install_time;
    char install_path[512];       // Where package is installed
} package_t;

typedef struct {
    char *data;
    size_t size;
    size_t capacity;
} download_buffer_t;

// Global configuration
struct nexuspkg_config {
    char repo_url[MAX_URL];
    char ai_models_repo[MAX_URL];
    char cache_path[256];
    char db_path[256];
    
    // Multi-format repository URLs
    char ubuntu_repo[MAX_URL];
    char debian_repo[MAX_URL];
    char arch_repo[MAX_URL];
    char fedora_repo[MAX_URL];
    char opensuse_repo[MAX_URL];
    char aur_repo[MAX_URL];
    char flatpak_repo[MAX_URL];
    char appimage_repo[MAX_URL];
    char snap_store[MAX_URL];
    
    int verbose;
    int auto_resolve_deps;
    int enable_ai_features;
    int parallel_downloads;
    int enable_deb_support;
    int enable_rpm_support;
    int enable_zst_support;
    int enable_appimage_support;
    int enable_flatpak_support;
    int enable_snap_support;
    int sandbox_untrusted;
    int auto_update_repos;
} config = {
    .repo_url = NEXUSPKG_REPO_URL,
    .ai_models_repo = NEXUSPKG_AI_MODELS_REPO,
    .cache_path = NEXUSPKG_CACHE_PATH,
    .db_path = NEXUSPKG_DB_PATH,
    
    // Default repository URLs
    .ubuntu_repo = "https://archive.ubuntu.com/ubuntu",
    .debian_repo = "https://deb.debian.org/debian",
    .arch_repo = "https://mirror.rackspace.com/archlinux",
    .fedora_repo = "https://download.fedoraproject.org/pub/fedora/linux",
    .opensuse_repo = "https://download.opensuse.org",
    .aur_repo = "https://aur.archlinux.org",
    .flatpak_repo = "https://dl.flathub.org/repo/flathub.flatpakrepo",
    .appimage_repo = "https://appimage.github.io",
    .snap_store = "https://api.snapcraft.io",
    
    .verbose = 0,
    .auto_resolve_deps = 1,
    .enable_ai_features = 1,
    .parallel_downloads = 4,
    .enable_deb_support = 1,
    .enable_rpm_support = 1,
    .enable_zst_support = 1,
    .enable_appimage_support = 1,
    .enable_flatpak_support = 1,
    .enable_snap_support = 1,
    .sandbox_untrusted = 1,
    .auto_update_repos = 1
};

// Database handle
static sqlite3 *db = NULL;

// Color output
#define COLOR_RED     "\033[0;31m"
#define COLOR_GREEN   "\033[0;32m"
#define COLOR_YELLOW  "\033[1;33m"
#define COLOR_BLUE    "\033[0;34m"
#define COLOR_PURPLE  "\033[0;35m"
#define COLOR_CYAN    "\033[0;36m"
#define COLOR_WHITE   "\033[1;37m"
#define COLOR_RESET   "\033[0m"

// Function prototypes
int nexuspkg_init(void);
void nexuspkg_cleanup(void);
int load_config(void);
int init_database(void);
int sync_repositories(void);
int install_package(const char *package_name);
int remove_package(const char *package_name);
int search_packages(const char *query);
int list_installed_packages(void);
int update_system(void);
int download_file(const char *url, const char *output_path, const char *expected_checksum);
int verify_checksum(const char *file_path, const char *expected_checksum);
int extract_package(const char *archive_path, const char *extract_path);
int resolve_dependencies(const char *package_name, char dependencies[][MAX_PACKAGE_NAME], int *count);
size_t download_callback(void *contents, size_t size, size_t nmemb, download_buffer_t *buffer);

// Format-specific installation functions
int install_deb_package(const char *package_path);
int install_rpm_package(const char *package_path);
int install_zst_package(const char *package_path);
int install_appimage_package(const char *package_path, const char *package_name);
int install_flatpak_package(const char *package_name);
int install_snap_package(const char *package_name);
int install_native_package(const char *package_path);
int install_tar_package(const char *package_path, const char *extract_path);
int install_zip_package(const char *package_path, const char *extract_path);
int install_python_wheel(const char *package_path);
int install_npm_package(const char *package_name);
int install_cargo_package(const char *package_name);

// Repository synchronization functions
int sync_deb_repository(const char *repo_url);
int sync_rpm_repository(const char *repo_url);
int sync_arch_repository(const char *repo_url);
int sync_flatpak_repository(void);
int sync_snap_repository(void);
int sync_appimage_repository(void);

// Package format detection
package_format_t detect_package_format(const char *filename);
const char* format_to_string(package_format_t format);
package_format_t string_to_format(const char *format_str);

// Print functions
void print_info(const char *format, ...) {
    printf(COLOR_BLUE "[INFO]" COLOR_RESET " ");
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
    printf("\n");
}

void print_success(const char *format, ...) {
    printf(COLOR_GREEN "[SUCCESS]" COLOR_RESET " ");
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
    printf("\n");
}

void print_warning(const char *format, ...) {
    printf(COLOR_YELLOW "[WARNING]" COLOR_RESET " ");
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
    printf("\n");
}

void print_error(const char *format, ...) {
    printf(COLOR_RED "[ERROR]" COLOR_RESET " ");
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
    printf("\n");
}

// Initialize NexusPkg
int nexuspkg_init(void) {
    print_info("Initializing NexusPkg v%s", NEXUSPKG_VERSION);
    
    // Create necessary directories
    mkdir(config.cache_path, 0755);
    mkdir("/var/lib/nexuspkg", 0755);
    mkdir("/var/log/nexuspkg", 0755);
    
    // Load configuration
    if (load_config() != 0) {
        print_warning("Failed to load config, using defaults");
    }
    
    // Initialize database
    if (init_database() != 0) {
        print_error("Failed to initialize database");
        return -1;
    }
    
    // Initialize curl
    curl_global_init(CURL_GLOBAL_DEFAULT);
    
    print_success("NexusPkg initialized successfully");
    return 0;
}

// Load configuration
int load_config(void) {
    FILE *fp = fopen(NEXUSPKG_CONFIG_PATH, "r");
    if (!fp) {
        return -1; // Use defaults
    }
    
    char line[512];
    while (fgets(line, sizeof(line), fp)) {
        // Remove newline
        line[strcspn(line, "\n")] = 0;
        
        // Skip comments and empty lines
        if (line[0] == '#' || line[0] == '\0') continue;
        
        // Parse key=value pairs
        char *key = strtok(line, "=");
        char *value = strtok(NULL, "=");
        
        if (!key || !value) continue;
        
        if (strcmp(key, "repo_url") == 0) {
            strncpy(config.repo_url, value, sizeof(config.repo_url) - 1);
        } else if (strcmp(key, "ai_models_repo") == 0) {
            strncpy(config.ai_models_repo, value, sizeof(config.ai_models_repo) - 1);
        } else if (strcmp(key, "verbose") == 0) {
            config.verbose = atoi(value);
        } else if (strcmp(key, "auto_resolve_deps") == 0) {
            config.auto_resolve_deps = atoi(value);
        } else if (strcmp(key, "enable_ai_features") == 0) {
            config.enable_ai_features = atoi(value);
        } else if (strcmp(key, "parallel_downloads") == 0) {
            config.parallel_downloads = atoi(value);
        }
    }
    
    fclose(fp);
    return 0;
}

// Initialize package database
int init_database(void) {
    int rc = sqlite3_open(config.db_path, &db);
    if (rc) {
        print_error("Cannot open database: %s", sqlite3_errmsg(db));
        return -1;
    }
    
    // Create packages table
    const char *sql = 
        "CREATE TABLE IF NOT EXISTS packages ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "name TEXT NOT NULL,"
        "version TEXT NOT NULL,"
        "description TEXT,"
        "download_url TEXT,"
        "checksum TEXT,"
        "size INTEGER,"
        "type INTEGER,"
        "format INTEGER DEFAULT 0,"
        "source_repo TEXT,"
        "architecture TEXT DEFAULT 'x86_64',"
        "dependencies TEXT,"
        "installed INTEGER DEFAULT 0,"
        "install_time INTEGER,"
        "install_path TEXT,"
        "UNIQUE(name, format, architecture)"
        ");";
    
    rc = sqlite3_exec(db, sql, 0, 0, NULL);
    if (rc != SQLITE_OK) {
        print_error("Failed to create packages table: %s", sqlite3_errmsg(db));
        return -1;
    }
    
    // Create repository cache table
    sql = "CREATE TABLE IF NOT EXISTS repo_cache ("
         "id INTEGER PRIMARY KEY AUTOINCREMENT,"
         "repo_url TEXT NOT NULL,"
         "last_sync INTEGER,"
         "package_count INTEGER"
         ");";
    
    rc = sqlite3_exec(db, sql, 0, 0, NULL);
    if (rc != SQLITE_OK) {
        print_error("Failed to create repo_cache table: %s", sqlite3_errmsg(db));
        return -1;
    }
    
    return 0;
}

// Download callback for curl
size_t download_callback(void *contents, size_t size, size_t nmemb, download_buffer_t *buffer) {
    size_t real_size = size * nmemb;
    
    // Resize buffer if needed
    while (buffer->size + real_size >= buffer->capacity) {
        buffer->capacity *= 2;
        buffer->data = realloc(buffer->data, buffer->capacity);
        if (!buffer->data) {
            return 0; // Out of memory
        }
    }
    
    memcpy(buffer->data + buffer->size, contents, real_size);
    buffer->size += real_size;
    
    return real_size;
}

// Sync package repositories
int sync_repositories(void) {
    print_info("Syncing package repositories...");
    
    CURL *curl = curl_easy_init();
    if (!curl) {
        print_error("Failed to initialize curl");
        return -1;
    }
    
    download_buffer_t buffer = {0};
    buffer.capacity = 4096;
    buffer.data = malloc(buffer.capacity);
    
    // Download package index
    char index_url[MAX_URL];
    snprintf(index_url, sizeof(index_url), "%s/index.json", config.repo_url);
    
    curl_easy_setopt(curl, CURLOPT_URL, index_url);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, download_callback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &buffer);
    curl_easy_setopt(curl, CURLOPT_USERAGENT, "NexusPkg/" NEXUSPKG_VERSION);
    
    CURLcode res = curl_easy_perform(curl);
    
    if (res != CURLE_OK) {
        print_error("Failed to download package index: %s", curl_easy_strerror(res));
        curl_easy_cleanup(curl);
        free(buffer.data);
        return -1;
    }
    
    // Parse JSON response
    json_object *root = json_tokener_parse(buffer.data);
    if (!root) {
        print_error("Failed to parse package index JSON");
        curl_easy_cleanup(curl);
        free(buffer.data);
        return -1;
    }
    
    json_object *packages;
    if (!json_object_object_get_ex(root, "packages", &packages)) {
        print_error("Invalid package index format");
        json_object_put(root);
        curl_easy_cleanup(curl);
        free(buffer.data);
        return -1;
    }
    
    // Clear existing repository data
    sqlite3_exec(db, "DELETE FROM packages WHERE installed = 0", NULL, NULL, NULL);
    
    // Process packages
    int array_len = json_object_array_length(packages);
    int processed = 0;
    
    for (int i = 0; i < array_len; i++) {
        json_object *pkg_obj = json_object_array_get_idx(packages, i);
        if (!pkg_obj) continue;
        
        json_object *name_obj, *version_obj, *desc_obj, *url_obj, *checksum_obj, *size_obj, *type_obj, *deps_obj;
        
        if (json_object_object_get_ex(pkg_obj, "name", &name_obj) &&
            json_object_object_get_ex(pkg_obj, "version", &version_obj) &&
            json_object_object_get_ex(pkg_obj, "description", &desc_obj) &&
            json_object_object_get_ex(pkg_obj, "download_url", &url_obj) &&
            json_object_object_get_ex(pkg_obj, "checksum", &checksum_obj) &&
            json_object_object_get_ex(pkg_obj, "size", &size_obj)) {
            
            const char *name = json_object_get_string(name_obj);
            const char *version = json_object_get_string(version_obj);
            const char *description = json_object_get_string(desc_obj);
            const char *download_url = json_object_get_string(url_obj);
            const char *checksum = json_object_get_string(checksum_obj);
            int64_t size = json_object_get_int64(size_obj);
            
            int type = 0;
            if (json_object_object_get_ex(pkg_obj, "type", &type_obj)) {
                type = json_object_get_int(type_obj);
            }
            
            // Handle dependencies
            char deps_str[1024] = "";
            if (json_object_object_get_ex(pkg_obj, "dependencies", &deps_obj) && 
                json_object_get_type(deps_obj) == json_type_array) {
                int deps_len = json_object_array_length(deps_obj);
                for (int j = 0; j < deps_len; j++) {
                    json_object *dep_obj = json_object_array_get_idx(deps_obj, j);
                    if (dep_obj) {
                        if (j > 0) strcat(deps_str, ",");
                        strcat(deps_str, json_object_get_string(dep_obj));
                    }
                }
            }
            
            // Insert into database
            const char *insert_sql = "INSERT OR REPLACE INTO packages "
                                   "(name, version, description, download_url, checksum, size, type, dependencies) "
                                   "VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
            
            sqlite3_stmt *stmt;
            if (sqlite3_prepare_v2(db, insert_sql, -1, &stmt, NULL) == SQLITE_OK) {
                sqlite3_bind_text(stmt, 1, name, -1, SQLITE_STATIC);
                sqlite3_bind_text(stmt, 2, version, -1, SQLITE_STATIC);
                sqlite3_bind_text(stmt, 3, description, -1, SQLITE_STATIC);
                sqlite3_bind_text(stmt, 4, download_url, -1, SQLITE_STATIC);
                sqlite3_bind_text(stmt, 5, checksum, -1, SQLITE_STATIC);
                sqlite3_bind_int64(stmt, 6, size);
                sqlite3_bind_int(stmt, 7, type);
                sqlite3_bind_text(stmt, 8, deps_str, -1, SQLITE_STATIC);
                
                if (sqlite3_step(stmt) == SQLITE_DONE) {
                    processed++;
                }
                
                sqlite3_finalize(stmt);
            }
        }
    }
    
    // Update sync time
    const char *update_sync_sql = "INSERT OR REPLACE INTO repo_cache "
                                 "(repo_url, last_sync, package_count) VALUES (?, ?, ?)";
    sqlite3_stmt *stmt;
    if (sqlite3_prepare_v2(db, update_sync_sql, -1, &stmt, NULL) == SQLITE_OK) {
        sqlite3_bind_text(stmt, 1, config.repo_url, -1, SQLITE_STATIC);
        sqlite3_bind_int64(stmt, 2, time(NULL));
        sqlite3_bind_int(stmt, 3, processed);
        sqlite3_step(stmt);
        sqlite3_finalize(stmt);
    }
    
    print_success("Synchronized %d packages from repository", processed);
    
    // Cleanup
    json_object_put(root);
    curl_easy_cleanup(curl);
    free(buffer.data);
    
    return 0;
}

// Search packages
int search_packages(const char *query) {
    if (!query) {
        print_error("Search query cannot be empty");
        return -1;
    }
    
    print_info("Searching for packages matching: %s", query);
    
    const char *search_sql = "SELECT name, version, description, type, installed FROM packages "
                           "WHERE name LIKE ? OR description LIKE ? ORDER BY name";
    
    sqlite3_stmt *stmt;
    int rc = sqlite3_prepare_v2(db, search_sql, -1, &stmt, NULL);
    if (rc != SQLITE_OK) {
        print_error("Failed to prepare search query: %s", sqlite3_errmsg(db));
        return -1;
    }
    
    char search_pattern[512];
    snprintf(search_pattern, sizeof(search_pattern), "%%%s%%", query);
    
    sqlite3_bind_text(stmt, 1, search_pattern, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 2, search_pattern, -1, SQLITE_STATIC);
    
    printf("\n" COLOR_WHITE "Search Results:" COLOR_RESET "\n");
    printf("%-30s %-15s %-10s %s\n", "Package", "Version", "Status", "Description");
    printf("================================================================================\n");
    
    int count = 0;
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        const char *name = (const char*)sqlite3_column_text(stmt, 0);
        const char *version = (const char*)sqlite3_column_text(stmt, 1);
        const char *description = (const char*)sqlite3_column_text(stmt, 2);
        int type = sqlite3_column_int(stmt, 3);
        int installed = sqlite3_column_int(stmt, 4);
        
        const char *status_color = installed ? COLOR_GREEN : COLOR_YELLOW;
        const char *status = installed ? "Installed" : "Available";
        
        const char *type_indicator = "";
        switch (type) {
            case PKG_TYPE_AI_MODEL: type_indicator = "🤖"; break;
            case PKG_TYPE_DATASET: type_indicator = "📊"; break;
            case PKG_TYPE_DESKTOP_APP: type_indicator = "🖥️"; break;
            case PKG_TYPE_LIBRARY: type_indicator = "📚"; break;
            default: type_indicator = "📦"; break;
        }
        
        printf("%-30s %-15s %s%-10s%s %s%s\n", 
               name, version, status_color, status, COLOR_RESET, type_indicator, description);
        count++;
    }
    
    sqlite3_finalize(stmt);
    
    if (count == 0) {
        print_info("No packages found matching '%s'", query);
    } else {
        print_success("Found %d packages matching '%s'", count, query);
    }
    
    return 0;
}

// Install package
int install_package(const char *package_name) {
    if (!package_name) {
        print_error("Package name cannot be empty");
        return -1;
    }
    
    print_info("Installing package: %s", package_name);
    
    // Check if already installed
    const char *check_sql = "SELECT name, version, installed FROM packages WHERE name = ?";
    sqlite3_stmt *stmt;
    int rc = sqlite3_prepare_v2(db, check_sql, -1, &stmt, NULL);
    if (rc != SQLITE_OK) {
        print_error("Failed to check package status: %s", sqlite3_errmsg(db));
        return -1;
    }
    
    sqlite3_bind_text(stmt, 1, package_name, -1, SQLITE_STATIC);
    
    if (sqlite3_step(stmt) != SQLITE_ROW) {
        print_error("Package '%s' not found in repository", package_name);
        sqlite3_finalize(stmt);
        return -1;
    }
    
    int installed = sqlite3_column_int(stmt, 2);
    if (installed) {
        const char *version = (const char*)sqlite3_column_text(stmt, 1);
        print_warning("Package '%s' version %s is already installed", package_name, version);
        sqlite3_finalize(stmt);
        return 0;
    }
    
    sqlite3_finalize(stmt);
    
    // Get package details including format
    const char *details_sql = "SELECT download_url, checksum, size, dependencies, format FROM packages WHERE name = ?";
    rc = sqlite3_prepare_v2(db, details_sql, -1, &stmt, NULL);
    if (rc != SQLITE_OK) {
        print_error("Failed to get package details: %s", sqlite3_errmsg(db));
        return -1;
    }
    
    sqlite3_bind_text(stmt, 1, package_name, -1, SQLITE_STATIC);
    
    if (sqlite3_step(stmt) != SQLITE_ROW) {
        print_error("Failed to retrieve package details");
        sqlite3_finalize(stmt);
        return -1;
    }
    
    const char *download_url = (const char*)sqlite3_column_text(stmt, 0);
    const char *checksum = (const char*)sqlite3_column_text(stmt, 1);
    int64_t size = sqlite3_column_int64(stmt, 2);
    const char *dependencies = (const char*)sqlite3_column_text(stmt, 3);
    package_format_t format = (package_format_t)sqlite3_column_int(stmt, 4);
    
    print_info("Package format: %s", format_to_string(format));
    
    // Handle format-specific installation
    int install_result = -1;
    
    switch (format) {
        case PKG_FORMAT_FLATPAK:
            install_result = install_flatpak_package(package_name);
            break;
            
        case PKG_FORMAT_SNAP:
            install_result = install_snap_package(package_name);
            break;
            
        case PKG_FORMAT_NPM:
            install_result = install_npm_package(package_name);
            break;
            
        case PKG_FORMAT_CARGO:
            install_result = install_cargo_package(package_name);
            break;
            
        default: {
            // Download-based installation for file packages
            char download_path[512];
            const char *ext = "";
            
            switch (format) {
                case PKG_FORMAT_DEB: ext = ".deb"; break;
                case PKG_FORMAT_RPM: ext = ".rpm"; break;
                case PKG_FORMAT_ZST: ext = ".pkg.tar.zst"; break;
                case PKG_FORMAT_APPIMAGE: ext = ".AppImage"; break;
                case PKG_FORMAT_TAR_XZ: ext = ".tar.xz"; break;
                case PKG_FORMAT_TAR_GZ: ext = ".tar.gz"; break;
                case PKG_FORMAT_ZIP: ext = ".zip"; break;
                case PKG_FORMAT_PYTHON_WHEEL: ext = ".whl"; break;
                default: ext = ".pkg"; break;
            }
            
            snprintf(download_path, sizeof(download_path), "%s/%s%s", config.cache_path, package_name, ext);
            
            // Download package
            print_info("Downloading %s (%.2f MB)...", package_name, (double)size / (1024 * 1024));
            if (download_file(download_url, download_path, checksum) != 0) {
                print_error("Failed to download package");
                sqlite3_finalize(stmt);
                return -1;
            }
            
            // Install based on format
            switch (format) {
                case PKG_FORMAT_DEB:
                    install_result = install_deb_package(download_path);
                    break;
                case PKG_FORMAT_RPM:
                    install_result = install_rpm_package(download_path);
                    break;
                case PKG_FORMAT_ZST:
                    install_result = install_zst_package(download_path);
                    break;
                case PKG_FORMAT_APPIMAGE:
                    install_result = install_appimage_package(download_path, package_name);
                    break;
                case PKG_FORMAT_PYTHON_WHEEL:
                    install_result = install_python_wheel(download_path);
                    break;
                case PKG_FORMAT_NATIVE:
                default: {
                    // Native package installation (extract and run script)
                    char extract_path[512];
                    snprintf(extract_path, sizeof(extract_path), "/tmp/nexuspkg-install-%s", package_name);
                    mkdir(extract_path, 0755);
                    
                    print_info("Extracting package...");
                    if (extract_package(download_path, extract_path) != 0) {
                        print_error("Failed to extract package");
                        sqlite3_finalize(stmt);
                        return -1;
                    }
                    
                    // Run installation script if present
                    char install_script[512];
                    snprintf(install_script, sizeof(install_script), "%s/install.sh", extract_path);
                    
                    struct stat script_stat;
                    if (stat(install_script, &script_stat) == 0 && (script_stat.st_mode & S_IXUSR)) {
                        print_info("Running installation script...");
                        char cmd[1024];
                        snprintf(cmd, sizeof(cmd), "cd %s && ./install.sh", extract_path);
                        
                        install_result = system(cmd);
                        if (install_result != 0) {
                            print_error("Installation script failed with code %d", install_result);
                        } else {
                            install_result = 0;
                        }
                    } else {
                        // No installation script, just copy files
                        char cmd[1024];
                        snprintf(cmd, sizeof(cmd), "cd %s && cp -r . /", extract_path);
                        install_result = system(cmd);
                    }
                    
                    // Cleanup
                    char cleanup_cmd[512];
                    snprintf(cleanup_cmd, sizeof(cleanup_cmd), "rm -rf %s", extract_path);
                    system(cleanup_cmd);
                    
                    break;
                }
            }
            break;
        }
    }
    
    if (install_result == 0) {
        // Mark as installed
        char install_path[512] = "/usr/local"; // Default install path
        
        const char *mark_installed_sql = "UPDATE packages SET installed = 1, install_time = ?, install_path = ? WHERE name = ?";
        sqlite3_stmt *update_stmt;
        rc = sqlite3_prepare_v2(db, mark_installed_sql, -1, &update_stmt, NULL);
        if (rc == SQLITE_OK) {
            sqlite3_bind_int64(update_stmt, 1, time(NULL));
            sqlite3_bind_text(update_stmt, 2, install_path, -1, SQLITE_STATIC);
            sqlite3_bind_text(update_stmt, 3, package_name, -1, SQLITE_STATIC);
            sqlite3_step(update_stmt);
            sqlite3_finalize(update_stmt);
        }
        
        print_success("Package '%s' installed successfully", package_name);
    } else {
        print_error("Failed to install package '%s'", package_name);
    }
    
    sqlite3_finalize(stmt);
    return install_result;
}

// Download file with progress and verification
int download_file(const char *url, const char *output_path, const char *expected_checksum) {
    CURL *curl = curl_easy_init();
    if (!curl) return -1;
    
    FILE *fp = fopen(output_path, "wb");
    if (!fp) {
        curl_easy_cleanup(curl);
        return -1;
    }
    
    curl_easy_setopt(curl, CURLOPT_URL, url);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, fp);
    curl_easy_setopt(curl, CURLOPT_USERAGENT, "NexusPkg/" NEXUSPKG_VERSION);
    curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
    curl_easy_setopt(curl, CURLOPT_FAILONERROR, 1L);
    
    CURLcode res = curl_easy_perform(curl);
    
    fclose(fp);
    curl_easy_cleanup(curl);
    
    if (res != CURLE_OK) {
        unlink(output_path);
        return -1;
    }
    
    // Verify checksum if provided
    if (expected_checksum && strlen(expected_checksum) > 0) {
        if (verify_checksum(output_path, expected_checksum) != 0) {
            print_error("Checksum verification failed");
            unlink(output_path);
            return -1;
        }
        print_success("Checksum verified");
    }
    
    return 0;
}

// Main function
int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("NexusPkg v%s - Universal AI-Native Package Manager for NexusOS\n", NEXUSPKG_VERSION);
        printf("Usage: %s <command> [options]\n\n", argv[0]);
        printf("" COLOR_CYAN "Core Commands:" COLOR_RESET "\n");
        printf("  sync              Synchronize all package repositories\n");
        printf("  search <query>    Search for packages across all formats\n");
        printf("  install <pkg>     Install a package (auto-detects format)\n");
        printf("  remove <pkg>      Remove an installed package\n");
        printf("  list              List all installed packages\n");
        printf("  update            Update all packages\n");
        printf("  info <pkg>        Show detailed package information\n");
        printf("  clean             Clean package cache and temp files\n");
        printf("\n");
        printf("" COLOR_YELLOW "Format-Specific Commands:" COLOR_RESET "\n");
        printf("  deb install <pkg> Install DEB package\n");
        printf("  rpm install <pkg> Install RPM package\n");
        printf("  zst install <pkg> Install Arch ZST package\n");
        printf("  flatpak <pkg>     Install Flatpak package\n");
        printf("  snap <pkg>        Install Snap package\n");
        printf("  appimage <file>   Install AppImage\n");
        printf("  pip <pkg>         Install Python package\n");
        printf("  npm <pkg>         Install NPM package\n");
        printf("  cargo <pkg>       Install Rust package\n");
        printf("\n");
        printf("" COLOR_GREEN "Repository Management:" COLOR_RESET "\n");
        printf("  repo add <url>    Add package repository\n");
        printf("  repo list         List configured repositories\n");
        printf("  repo sync <name>  Sync specific repository\n");
        printf("\n");
        printf("" COLOR_PURPLE "AI Features:" COLOR_RESET "\n");
        printf("  ai-search <desc>  AI-powered semantic package search\n");
        printf("  ai-recommend      Get AI package recommendations\n");
        printf("  ai-analyze        Analyze system for missing packages\n");
        printf("  models sync       Sync AI model repository\n");
        printf("  models install    Install AI models\n");
        printf("\n");
        printf("" COLOR_WHITE "Supported Formats:" COLOR_RESET "\n");
        printf("  📦 Native (.npkg)    🐧 DEB (.deb)         📦 RPM (.rpm)\n");
        printf("  🗜️  ZST (.pkg.tar.zst) 📱 AppImage (.AppImage) 📦 Flatpak\n");
        printf("  📦 Snap              🐍 Python Wheel       📦 NPM\n");
        printf("  🦀 Cargo             🐳 Docker             📦 OCI\n");
        printf("  📁 TAR.XZ/TAR.GZ      📁 ZIP               ⚙️  Binary\n");
        return 1;
    }
    
    // Initialize package manager
    if (nexuspkg_init() != 0) {
        return 1;
    }
    
    // Parse commands
    const char *command = argv[1];
    int result = 0;
    
    if (strcmp(command, "sync") == 0) {
        result = sync_repositories();
    } else if (strcmp(command, "search") == 0) {
        if (argc < 3) {
            print_error("Search query required");
            result = 1;
        } else {
            result = search_packages(argv[2]);
        }
    } else if (strcmp(command, "install") == 0) {
        if (argc < 3) {
            print_error("Package name required");
            result = 1;
        } else {
            result = install_package(argv[2]);
        }
    } else if (strcmp(command, "list") == 0) {
        result = list_installed_packages();
    } 
    // Format-specific installations
    else if (strcmp(command, "deb") == 0 && argc >= 4 && strcmp(argv[2], "install") == 0) {
        result = install_deb_package(argv[3]);
    } else if (strcmp(command, "rpm") == 0 && argc >= 4 && strcmp(argv[2], "install") == 0) {
        result = install_rpm_package(argv[3]);
    } else if (strcmp(command, "zst") == 0 && argc >= 4 && strcmp(argv[2], "install") == 0) {
        result = install_zst_package(argv[3]);
    } else if (strcmp(command, "flatpak") == 0 && argc >= 3) {
        result = install_flatpak_package(argv[2]);
    } else if (strcmp(command, "snap") == 0 && argc >= 3) {
        result = install_snap_package(argv[2]);
    } else if (strcmp(command, "appimage") == 0 && argc >= 3) {
        char *basename = strrchr(argv[2], '/');
        basename = basename ? basename + 1 : argv[2];
        // Remove .AppImage extension for package name
        char pkg_name[256];
        strncpy(pkg_name, basename, sizeof(pkg_name) - 1);
        char *dot = strstr(pkg_name, ".AppImage");
        if (dot) *dot = '\0';
        result = install_appimage_package(argv[2], pkg_name);
    } else if (strcmp(command, "pip") == 0 && argc >= 3) {
        result = install_python_wheel(argv[2]);
    } else if (strcmp(command, "npm") == 0 && argc >= 3) {
        result = install_npm_package(argv[2]);
    } else if (strcmp(command, "cargo") == 0 && argc >= 3) {
        result = install_cargo_package(argv[2]);
    } 
    // Status and information
    else if (strcmp(command, "status") == 0) {
        printf("\n" COLOR_BLUE "NexusPkg System Status:" COLOR_RESET "\n");
        printf("Version: %s\n", NEXUSPKG_VERSION);
        printf("Database: %s\n", config.db_path);
        printf("Cache: %s\n", config.cache_path);
        printf("\nFormat Support:\n");
        printf("  DEB: %s\n", config.enable_deb_support ? "✓ Enabled" : "✗ Disabled");
        printf("  RPM: %s\n", config.enable_rpm_support ? "✓ Enabled" : "✗ Disabled");
        printf("  ZST: %s\n", config.enable_zst_support ? "✓ Enabled" : "✗ Disabled");
        printf("  Flatpak: %s\n", config.enable_flatpak_support ? "✓ Enabled" : "✗ Disabled");
        printf("  Snap: %s\n", config.enable_snap_support ? "✓ Enabled" : "✗ Disabled");
        printf("  AppImage: %s\n", config.enable_appimage_support ? "✓ Enabled" : "✗ Disabled");
        printf("  AI Features: %s\n", config.enable_ai_features ? "✓ Enabled" : "✗ Disabled");
        result = 0;
    } else {
        print_error("Unknown command: %s", command);
        print_info("Use '%s' without arguments to see available commands", argv[0]);
        result = 1;
    }
    
    nexuspkg_cleanup();
    return result;
}

// Package format detection
package_format_t detect_package_format(const char *filename) {
    if (!filename) return PKG_FORMAT_NATIVE;
    
    if (strstr(filename, ".deb")) return PKG_FORMAT_DEB;
    if (strstr(filename, ".rpm")) return PKG_FORMAT_RPM;
    if (strstr(filename, ".pkg.tar.zst")) return PKG_FORMAT_ZST;
    if (strstr(filename, ".pkg.tar.xz")) return PKG_FORMAT_ZST;
    if (strstr(filename, ".AppImage")) return PKG_FORMAT_APPIMAGE;
    if (strstr(filename, ".flatpak")) return PKG_FORMAT_FLATPAK;
    if (strstr(filename, ".snap")) return PKG_FORMAT_SNAP;
    if (strstr(filename, ".tar.xz")) return PKG_FORMAT_TAR_XZ;
    if (strstr(filename, ".tar.gz")) return PKG_FORMAT_TAR_GZ;
    if (strstr(filename, ".zip")) return PKG_FORMAT_ZIP;
    if (strstr(filename, ".whl")) return PKG_FORMAT_PYTHON_WHEEL;
    if (strstr(filename, ".npkg")) return PKG_FORMAT_NATIVE;
    
    return PKG_FORMAT_BINARY; // Default to binary
}

const char* format_to_string(package_format_t format) {
    switch (format) {
        case PKG_FORMAT_NATIVE: return "native";
        case PKG_FORMAT_DEB: return "deb";
        case PKG_FORMAT_RPM: return "rpm";
        case PKG_FORMAT_ZST: return "zst";
        case PKG_FORMAT_APPIMAGE: return "appimage";
        case PKG_FORMAT_FLATPAK: return "flatpak";
        case PKG_FORMAT_SNAP: return "snap";
        case PKG_FORMAT_TAR_XZ: return "tar.xz";
        case PKG_FORMAT_TAR_GZ: return "tar.gz";
        case PKG_FORMAT_ZIP: return "zip";
        case PKG_FORMAT_BINARY: return "binary";
        case PKG_FORMAT_PYTHON_WHEEL: return "wheel";
        case PKG_FORMAT_NPM: return "npm";
        case PKG_FORMAT_CARGO: return "cargo";
        case PKG_FORMAT_DOCKER: return "docker";
        case PKG_FORMAT_OCI: return "oci";
        default: return "unknown";
    }
}

// Install DEB package using dpkg
int install_deb_package(const char *package_path) {
    if (!config.enable_deb_support) {
        print_error("DEB package support is disabled");
        return -1;
    }
    
    print_info("Installing DEB package: %s", package_path);
    
    // Check if dpkg is available
    if (system("which dpkg > /dev/null 2>&1") != 0) {
        print_error("dpkg not found. Installing dpkg...");
        // Try to install dpkg first
        if (system("pacman -S --noconfirm dpkg") != 0) {
            print_error("Failed to install dpkg");
            return -1;
        }
    }
    
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "dpkg -i '%s'", package_path);
    
    int result = system(cmd);
    if (result != 0) {
        print_warning("dpkg installation failed, trying to fix dependencies...");
        system("apt-get install -f");
        return -1;
    }
    
    print_success("DEB package installed successfully");
    return 0;
}

// Install RPM package using rpm or alien
int install_rpm_package(const char *package_path) {
    if (!config.enable_rpm_support) {
        print_error("RPM package support is disabled");
        return -1;
    }
    
    print_info("Installing RPM package: %s", package_path);
    
    // Try rpm first, then alien as fallback
    if (system("which rpm > /dev/null 2>&1") == 0) {
        char cmd[1024];
        snprintf(cmd, sizeof(cmd), "rpm -i '%s'", package_path);
        
        if (system(cmd) == 0) {
            print_success("RPM package installed successfully");
            return 0;
        }
    }
    
    // Fallback to alien (convert RPM to DEB)
    if (system("which alien > /dev/null 2>&1") != 0) {
        print_info("Installing alien for RPM conversion...");
        if (system("pacman -S --noconfirm alien") != 0) {
            print_error("Failed to install alien");
            return -1;
        }
    }
    
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "alien -d '%s' && dpkg -i *.deb", package_path);
    
    if (system(cmd) == 0) {
        print_success("RPM package converted and installed successfully");
        return 0;
    }
    
    print_error("Failed to install RPM package");
    return -1;
}

// Install Arch Linux ZST package using pacman or manual extraction
int install_zst_package(const char *package_path) {
    if (!config.enable_zst_support) {
        print_error("ZST package support is disabled");
        return -1;
    }
    
    print_info("Installing ZST package: %s", package_path);
    
    // Try pacman first (native Arch)
    if (system("which pacman > /dev/null 2>&1") == 0) {
        char cmd[1024];
        snprintf(cmd, sizeof(cmd), "pacman -U --noconfirm '%s'", package_path);
        
        if (system(cmd) == 0) {
            print_success("ZST package installed successfully");
            return 0;
        }
    }
    
    // Manual extraction fallback
    print_info("Extracting ZST package manually...");
    
    char extract_dir[512];
    snprintf(extract_dir, sizeof(extract_dir), "/tmp/nexuspkg-zst-extract");
    mkdir(extract_dir, 0755);
    
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "cd '%s' && tar -xf '%s'", extract_dir, package_path);
    
    if (system(cmd) != 0) {
        print_error("Failed to extract ZST package");
        return -1;
    }
    
    // Copy files to root filesystem
    snprintf(cmd, sizeof(cmd), "cd '%s' && cp -r . /", extract_dir);
    
    if (system(cmd) == 0) {
        print_success("ZST package extracted and installed");
        return 0;
    }
    
    print_error("Failed to install ZST package");
    return -1;
}

// Install AppImage package
int install_appimage_package(const char *package_path, const char *package_name) {
    if (!config.enable_appimage_support) {
        print_error("AppImage support is disabled");
        return -1;
    }
    
    print_info("Installing AppImage: %s", package_name);
    
    // Create AppImages directory
    char appimage_dir[512];
    snprintf(appimage_dir, sizeof(appimage_dir), "/opt/appimages");
    mkdir(appimage_dir, 0755);
    
    // Copy AppImage to system location
    char dest_path[512];
    snprintf(dest_path, sizeof(dest_path), "%s/%s.AppImage", appimage_dir, package_name);
    
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "cp '%s' '%s' && chmod +x '%s'", 
             package_path, dest_path, dest_path);
    
    if (system(cmd) != 0) {
        print_error("Failed to install AppImage");
        return -1;
    }
    
    // Create desktop entry if possible
    snprintf(cmd, sizeof(cmd), "%s --appimage-extract-and-run --appimage-help > /dev/null 2>&1", dest_path);
    system(cmd); // Try to extract desktop integration
    
    // Create symlink in /usr/local/bin
    char symlink_path[512];
    snprintf(symlink_path, sizeof(symlink_path), "/usr/local/bin/%s", package_name);
    snprintf(cmd, sizeof(cmd), "ln -sf '%s' '%s'", dest_path, symlink_path);
    system(cmd);
    
    print_success("AppImage installed successfully");
    return 0;
}

// Install Flatpak package
int install_flatpak_package(const char *package_name) {
    if (!config.enable_flatpak_support) {
        print_error("Flatpak support is disabled");
        return -1;
    }
    
    print_info("Installing Flatpak package: %s", package_name);
    
    // Ensure flatpak is installed
    if (system("which flatpak > /dev/null 2>&1") != 0) {
        print_info("Installing Flatpak...");
        if (system("pacman -S --noconfirm flatpak") != 0) {
            print_error("Failed to install Flatpak");
            return -1;
        }
    }
    
    // Add Flathub repository if not exists
    system("flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo");
    
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "flatpak install -y flathub '%s'", package_name);
    
    if (system(cmd) == 0) {
        print_success("Flatpak package installed successfully");
        return 0;
    }
    
    print_error("Failed to install Flatpak package");
    return -1;
}

// Install Snap package
int install_snap_package(const char *package_name) {
    if (!config.enable_snap_support) {
        print_error("Snap support is disabled");
        return -1;
    }
    
    print_info("Installing Snap package: %s", package_name);
    
    // Ensure snapd is installed and running
    if (system("which snap > /dev/null 2>&1") != 0) {
        print_info("Installing snapd...");
        if (system("pacman -S --noconfirm snapd") != 0) {
            print_error("Failed to install snapd");
            return -1;
        }
        
        // Enable and start snapd
        system("systemctl enable --now snapd.socket");
        system("systemctl enable --now snapd.service");
    }
    
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "snap install '%s'", package_name);
    
    if (system(cmd) == 0) {
        print_success("Snap package installed successfully");
        return 0;
    }
    
    print_error("Failed to install Snap package");
    return -1;
}

// Install Python wheel package
int install_python_wheel(const char *package_path) {
    print_info("Installing Python wheel: %s", package_path);
    
    // Ensure pip is available
    if (system("which pip > /dev/null 2>&1") != 0) {
        if (system("which pip3 > /dev/null 2>&1") != 0) {
            print_info("Installing pip...");
            if (system("pacman -S --noconfirm python-pip") != 0) {
                print_error("Failed to install pip");
                return -1;
            }
        }
    }
    
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "pip install '%s'", package_path);
    
    if (system(cmd) == 0) {
        print_success("Python wheel installed successfully");
        return 0;
    }
    
    print_error("Failed to install Python wheel");
    return -1;
}

// Install NPM package
int install_npm_package(const char *package_name) {
    print_info("Installing NPM package: %s", package_name);
    
    // Ensure npm is available
    if (system("which npm > /dev/null 2>&1") != 0) {
        print_info("Installing npm...");
        if (system("pacman -S --noconfirm npm") != 0) {
            print_error("Failed to install npm");
            return -1;
        }
    }
    
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "npm install -g '%s'", package_name);
    
    if (system(cmd) == 0) {
        print_success("NPM package installed successfully");
        return 0;
    }
    
    print_error("Failed to install NPM package");
    return -1;
}

// Install Cargo package
int install_cargo_package(const char *package_name) {
    print_info("Installing Cargo package: %s", package_name);
    
    // Ensure cargo is available
    if (system("which cargo > /dev/null 2>&1") != 0) {
        print_info("Installing cargo...");
        if (system("pacman -S --noconfirm rust") != 0) {
            print_error("Failed to install cargo");
            return -1;
        }
    }
    
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "cargo install '%s'", package_name);
    
    if (system(cmd) == 0) {
        print_success("Cargo package installed successfully");
        return 0;
    }
    
    print_error("Failed to install Cargo package");
    return -1;
}

// Verify package checksum
int verify_checksum(const char *file_path, const char *expected_checksum) {
    if (!file_path || !expected_checksum) return -1;
    
    FILE *file = fopen(file_path, "rb");
    if (!file) return -1;
    
    unsigned char hash[SHA256_DIGEST_LENGTH];
    SHA256_CTX sha256;
    SHA256_Init(&sha256);
    
    char buffer[8192];
    size_t bytes;
    while ((bytes = fread(buffer, 1, sizeof(buffer), file)) > 0) {
        SHA256_Update(&sha256, buffer, bytes);
    }
    
    SHA256_Final(hash, &sha256);
    fclose(file);
    
    // Convert hash to hex string
    char calculated_checksum[65];
    for (int i = 0; i < SHA256_DIGEST_LENGTH; i++) {
        sprintf(&calculated_checksum[i * 2], "%02x", hash[i]);
    }
    calculated_checksum[64] = '\0';
    
    return (strcmp(calculated_checksum, expected_checksum) == 0) ? 0 : -1;
}

// Extract package using libarchive
int extract_package(const char *archive_path, const char *extract_path) {
    struct archive *a;
    struct archive *ext;
    struct archive_entry *entry;
    int flags;
    int r;
    
    /* Select which attributes we want to restore. */
    flags = ARCHIVE_EXTRACT_TIME;
    flags |= ARCHIVE_EXTRACT_PERM;
    flags |= ARCHIVE_EXTRACT_ACL;
    flags |= ARCHIVE_EXTRACT_FFLAGS;
    
    a = archive_read_new();
    archive_read_support_format_all(a);
    archive_read_support_compression_all(a);
    
    ext = archive_write_disk_new();
    archive_write_disk_set_options(ext, flags);
    archive_write_disk_set_standard_lookup(ext);
    
    if ((r = archive_read_open_filename(a, archive_path, 10240))) {
        archive_read_free(a);
        archive_write_free(ext);
        return -1;
    }
    
    while (archive_read_next_header(a, &entry) == ARCHIVE_OK) {
        const char *current_file = archive_entry_pathname(entry);
        char full_path[1024];
        snprintf(full_path, sizeof(full_path), "%s/%s", extract_path, current_file);
        archive_entry_set_pathname(entry, full_path);
        
        r = archive_write_header(ext, entry);
        if (r < ARCHIVE_OK) {
            continue;
        } else if (archive_entry_size(entry) > 0) {
            const void *buff;
            size_t size;
            la_int64_t offset;
            
            while ((r = archive_read_data_block(a, &buff, &size, &offset)) == ARCHIVE_OK) {
                archive_write_data_block(ext, buff, size, offset);
            }
            if (r < ARCHIVE_OK) {
                continue;
            }
        }
        archive_write_finish_entry(ext);
    }
    
    archive_read_close(a);
    archive_read_free(a);
    archive_write_close(ext);
    archive_write_free(ext);
    
    return 0;
}

// List installed packages
int list_installed_packages(void) {
    print_info("Listing installed packages...");
    
    const char *list_sql = "SELECT name, version, format, install_time, install_path FROM packages WHERE installed = 1 ORDER BY name";
    
    sqlite3_stmt *stmt;
    int rc = sqlite3_prepare_v2(db, list_sql, -1, &stmt, NULL);
    if (rc != SQLITE_OK) {
        print_error("Failed to prepare list query: %s", sqlite3_errmsg(db));
        return -1;
    }
    
    printf("\n" COLOR_WHITE "Installed Packages:" COLOR_RESET "\n");
    printf("%-30s %-15s %-12s %-20s %s\n", "Package", "Version", "Format", "Install Date", "Path");
    printf("==========================================================================================================\n");
    
    int count = 0;
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        const char *name = (const char*)sqlite3_column_text(stmt, 0);
        const char *version = (const char*)sqlite3_column_text(stmt, 1);
        package_format_t format = (package_format_t)sqlite3_column_int(stmt, 2);
        int64_t install_time = sqlite3_column_int64(stmt, 3);
        const char *install_path = (const char*)sqlite3_column_text(stmt, 4);
        
        time_t time = (time_t)install_time;
        char time_str[64];
        strftime(time_str, sizeof(time_str), "%Y-%m-%d %H:%M", localtime(&time));
        
        printf("%-30s %-15s %-12s %-20s %s\n", 
               name, version, format_to_string(format), time_str, 
               install_path ? install_path : "N/A");
        count++;
    }
    
    sqlite3_finalize(stmt);
    
    if (count == 0) {
        print_info("No packages installed");
    } else {
        print_success("%d packages installed", count);
    }
    
    return 0;
}

// Cleanup function
void nexuspkg_cleanup(void) {
    if (db) {
        sqlite3_close(db);
    }
    curl_global_cleanup();
}
