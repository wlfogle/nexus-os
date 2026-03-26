#!/bin/bash

# Ollama Code Checker - Interactive Code Analysis Tool
# Uses your existing Ollama models to analyze code for errors and improvements

set -e

# Configuration
MODELS_PATH="/run/media/garuda/73cf9511-0af0-4ac4-9d83-ee21eb17ff5d/models"
CONFIG_FILE="$HOME/.ollama-code-checker.conf"
TEMP_DIR="/tmp/ollama-analysis"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default configuration
DEFAULT_MODEL="granite-code:latest"
DEFAULT_ANALYSIS_TYPES="errors,style,security,performance"
DEFAULT_FILE_EXTENSIONS=".rs,.ts,.tsx,.js,.jsx,.py,.go,.java,.cpp,.c,.h"

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  Ollama Code Analysis Tool${NC}"
    echo -e "${BLUE}================================${NC}"
    echo
}

print_usage() {
    cat << EOF
Usage: $0 [OPTIONS] [DIRECTORY]

OPTIONS:
    -m, --model MODEL       Specify Ollama model to use
    -t, --type TYPE         Analysis type: errors,style,security,performance,all
    -e, --extensions EXT    File extensions to analyze (comma-separated)
    -r, --recursive         Analyze recursively (default)
    -f, --file FILE         Analyze single file
    --limit LIMIT           Maximum files to process (default: unlimited)
    -c, --config            Show current configuration
    -l, --list-models       List available models
    -i, --interactive       Interactive mode (default if no args)
    -o, --output FILE       Save results to file
    -v, --verbose           Verbose output
    -h, --help              Show this help

EXAMPLES:
    $0                                          # Interactive mode
    $0 /path/to/code                           # Analyze directory
    $0 -f main.rs                              # Analyze single file
    $0 -m deepseek-coder-v2:latest -t errors  # Use specific model for errors
    $0 --list-models                           # Show available models
    
ANALYSIS TYPES:
    errors      - Syntax errors, type errors, logic issues
    style       - Code style, formatting, best practices
    security    - Security vulnerabilities, unsafe patterns
    performance - Performance issues, optimization suggestions
    cleanup     - Stub code, unused functions, zombie code, dead imports
    all         - All of the above
EOF
}

load_config() {
    if [[ -f "$CONFIG_FILE" ]]; then
        source "$CONFIG_FILE"
    else
        # Create default config
        cat > "$CONFIG_FILE" << EOF
# Ollama Code Checker Configuration
MODEL="$DEFAULT_MODEL"
ANALYSIS_TYPES="$DEFAULT_ANALYSIS_TYPES"
FILE_EXTENSIONS="$DEFAULT_FILE_EXTENSIONS"
RECURSIVE=true
VERBOSE=false
EOF
    fi
}

save_config() {
    cat > "$CONFIG_FILE" << EOF
# Ollama Code Checker Configuration
MODEL="$MODEL"
ANALYSIS_TYPES="$ANALYSIS_TYPES"
FILE_EXTENSIONS="$FILE_EXTENSIONS"
RECURSIVE=$RECURSIVE
VERBOSE=$VERBOSE
EOF
}

ensure_ollama_running() {
    # Check if models directory exists
    if [[ ! -d "$MODELS_PATH" ]]; then
        echo -e "${RED}Error: Models directory not found at $MODELS_PATH${NC}"
        return 1
    fi
    
    export OLLAMA_MODELS="$MODELS_PATH"
    
    # Check if Ollama is already running and responsive
    if OLLAMA_MODELS="$MODELS_PATH" ollama list >/dev/null 2>&1; then
        echo -e "${GREEN}Ollama service is already running with correct models path${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}Checking Ollama service...${NC}"
    echo -e "${CYAN}Using models from: $MODELS_PATH${NC}"
    
    # Stop any existing ollama processes
    pkill -f "ollama serve" 2>/dev/null || true
    sleep 2
    
    echo -e "${YELLOW}Starting Ollama service with GPU acceleration and custom models path...${NC}"
    
    # Check if NVIDIA GPU is available
    if command -v nvidia-smi >/dev/null 2>&1; then
        echo -e "${CYAN}Using NVIDIA GPU acceleration...${NC}"
        # Force GPU usage and set memory limit
        export CUDA_VISIBLE_DEVICES=0
        export OLLAMA_NUM_PARALLEL=1
        export OLLAMA_MAX_LOADED_MODELS=1
    fi
    
    OLLAMA_MODELS="$MODELS_PATH" ollama serve > /dev/null 2>&1 &
    local ollama_pid=$!
    
    # Wait for service to start
    echo -e "${YELLOW}Waiting for Ollama to start...${NC}"
    for i in {1..10}; do
        if OLLAMA_MODELS="$MODELS_PATH" ollama list >/dev/null 2>&1; then
            echo -e "${GREEN}Ollama service started successfully (PID: $ollama_pid)${NC}"
            return 0
        fi
        sleep 1
        echo -n "."
    done
    
    echo -e "${RED}Failed to start Ollama service${NC}"
    return 1
}

list_available_models() {
    echo -e "${CYAN}Available Ollama Models:${NC}"
    echo "========================"
    
    # Ensure Ollama is running with correct models path
    ensure_ollama_running
    
    export OLLAMA_MODELS="$MODELS_PATH"
    models=$(OLLAMA_MODELS="$MODELS_PATH" ollama list 2>/dev/null | tail -n +2 | awk '{print $1}' | grep -v "^$" | grep -v "NAME")
    
    if [[ -z "$models" ]] || [[ "$models" == "NAME" ]]; then
        echo "No models found via 'ollama list'. Checking manifest directories..."
        if [[ -d "$MODELS_PATH/manifests/registry.ollama.ai/library" ]]; then
            # List code-focused models first
            echo -e "${YELLOW}Code Analysis Models (Recommended):${NC}"
            ls "$MODELS_PATH/manifests/registry.ollama.ai/library/" 2>/dev/null | grep -E "(code|granite|deepseek)" | while read -r model_name; do
                echo -e "${GREEN}● $model_name:latest${NC}"
            done
            echo
            echo -e "${YELLOW}Other Available Models:${NC}"
            ls "$MODELS_PATH/manifests/registry.ollama.ai/library/" 2>/dev/null | grep -vE "(code|granite|deepseek)" | while read -r model_name; do
                echo -e "  $model_name:latest"
            done
        else
            echo -e "${YELLOW}No models manifest found. You may need to pull some models first.${NC}"
        fi
    else
        echo "$models" | while read -r model; do
            if [[ "$model" == *"code"* ]] || [[ "$model" == *"granite"* ]] || [[ "$model" == *"deepseek"* ]]; then
                echo -e "${GREEN}● $model${NC} (recommended for code analysis)"
            else
                echo -e "  $model"
            fi
        done
    fi
    echo
    return 0
}

get_file_content() {
    local file="$1"
    local max_lines=500
    
    # Get file info
    local size=$(wc -l < "$file" 2>/dev/null || echo "0")
    
    if [[ $size -gt $max_lines ]]; then
        echo "# File: $file (showing first $max_lines lines of $size total)"
        head -n $max_lines "$file"
        echo -e "\n# ... (truncated, $((size - max_lines)) more lines)"
    else
        echo "# File: $file"
        cat "$file"
    fi
}

analyze_file() {
    local file="$1"
    local analysis_type="$2"
    
    echo -e "${YELLOW}Analyzing: $file${NC}"
    
    # Get file extension
    local ext="${file##*.}"
    local language=""
    
    case "$ext" in
        rs) language="Rust" ;;
        ts|tsx) language="TypeScript" ;;
        js|jsx) language="JavaScript" ;;
        py) language="Python" ;;
        go) language="Go" ;;
        java) language="Java" ;;
        cpp|cc|cxx) language="C++" ;;
        c) language="C" ;;
        h|hpp) language="C/C++ Header" ;;
        *) language="Unknown" ;;
    esac
    
    # Create prompt based on analysis type
    local prompt=""
    case "$analysis_type" in
        "errors")
            prompt="You are a code analysis expert. Analyze this $language code for syntax errors, type errors, logical issues, and potential bugs. Be specific about line numbers and provide clear explanations.

$(get_file_content "$file")

Please identify:
1. Syntax errors
2. Type errors
3. Logical issues
4. Potential runtime errors
5. Missing imports or dependencies

Format your response with clear sections and specific line references."
            ;;
        "style")
            prompt="You are a code style expert. Analyze this $language code for style issues, formatting problems, and adherence to best practices.

$(get_file_content "$file")

Please review:
1. Code formatting and indentation
2. Naming conventions
3. Code organization
4. Documentation quality
5. Best practice compliance

Provide specific suggestions for improvement."
            ;;
        "security")
            prompt="You are a security expert. Analyze this $language code for security vulnerabilities, unsafe patterns, and potential attack vectors.

$(get_file_content "$file")

Please identify:
1. Security vulnerabilities
2. Unsafe operations
3. Input validation issues
4. Authentication/authorization problems
5. Data exposure risks

Prioritize findings by severity level."
            ;;
        "performance")
            prompt="You are a performance optimization expert. Analyze this $language code for performance issues and optimization opportunities.

$(get_file_content "$file")

Please identify:
1. Performance bottlenecks
2. Inefficient algorithms
3. Memory usage issues
4. I/O optimization opportunities
5. Compilation/build optimizations

Suggest specific improvements with expected impact."
            ;;
        "cleanup")
            prompt="You are a code cleanup expert. Analyze this $language code for stub code, unused functions, zombie code, and dead imports that can be safely removed.

$(get_file_content "$file")

Please identify:
1. Stub functions (empty or placeholder implementations)
2. Unused functions, variables, and imports
3. Dead code that's never called or referenced
4. Commented-out code that should be removed
5. Deprecated patterns or legacy code
6. Redundant or duplicate code
7. Unused type definitions, interfaces, or structs
8. Empty catch blocks or TODO comments that need attention

For each finding, specify:
- Exact line numbers
- Why it's considered unused/stub/zombie
- Whether it's safe to remove
- Any dependencies to check before removal

Prioritize findings by cleanup impact and safety of removal."
            ;;
        *)
            prompt="You are a comprehensive code analysis expert. Analyze this $language code for errors, style issues, security vulnerabilities, performance problems, and cleanup opportunities.

$(get_file_content "$file")

Please provide a comprehensive analysis covering:
1. Errors and bugs
2. Code style and best practices
3. Security considerations
4. Performance optimization opportunities
5. Code cleanup (stub/unused/zombie code)
6. General improvements

Structure your response clearly with priorities and specific recommendations."
            ;;
    esac
    
    # Run analysis with Ollama
    export OLLAMA_MODELS="$MODELS_PATH"
    
    echo -e "${CYAN}Running analysis with model: $MODEL${NC}"
    echo "================================"
    
    # Create temporary file for analysis
    local temp_file="$TEMP_DIR/analysis_$(basename "$file")_$(date +%s).md"
    mkdir -p "$TEMP_DIR"
    
    {
        echo "# Code Analysis Report"
        echo "**File:** $file"
        echo "**Language:** $language"
        echo "**Analysis Type:** $analysis_type"
        echo "**Model:** $MODEL"
        echo "**Date:** $(date)"
        echo ""
        
        if timeout 300 env OLLAMA_MODELS="$MODELS_PATH" CUDA_VISIBLE_DEVICES=0 ollama run "$MODEL" "$prompt" 2>/dev/null; then
            echo ""
            echo "---"
            echo "Analysis completed successfully."
        else
            echo "Error: Analysis failed or timed out."
            return 1
        fi
    } | tee "$temp_file"
    
    echo -e "${GREEN}Analysis saved to: $temp_file${NC}"
    return 0
}

analyze_directory() {
    local dir="$1"
    local analysis_type="$2"
    
    if [[ ! -d "$dir" ]]; then
        echo -e "${RED}Error: Directory '$dir' does not exist.${NC}"
        return 1
    fi
    
    echo -e "${BLUE}Analyzing directory: $dir${NC}"
    echo -e "${BLUE}Analysis type: $analysis_type${NC}"
    echo -e "${BLUE}Model: $MODEL${NC}"
    echo
    
    # Build find command for file extensions
    local find_cmd="find '$dir'"
    if [[ "$RECURSIVE" == "true" ]]; then
        find_cmd="$find_cmd -type f"
    else
        find_cmd="$find_cmd -maxdepth 1 -type f"
    fi
    
    # Add extension filters
    local ext_filter=""
    IFS=',' read -ra EXTENSIONS <<< "$FILE_EXTENSIONS"
    for ext in "${EXTENSIONS[@]}"; do
        ext=$(echo "$ext" | sed 's/^[[:space:]]*\.//')
        if [[ -z "$ext_filter" ]]; then
            ext_filter="-name '*.$ext'"
        else
            ext_filter="$ext_filter -o -name '*.$ext'"
        fi
    done
    
    if [[ -n "$ext_filter" ]]; then
        find_cmd="$find_cmd \( $ext_filter \)"
    fi
    
    # Execute find and analyze files (exclude common build/dependency directories)
    find_cmd="$find_cmd -not -path '*/node_modules/*' -not -path '*/target/*' -not -path '*/build/*' -not -path '*/dist/*' -not -path '*/.git/*' -not -path '*/__pycache__/*'"
    local files=$(eval "$find_cmd" 2>/dev/null)
    local file_count=$(echo "$files" | wc -l)
    
    if [[ -z "$files" ]]; then
        echo -e "${YELLOW}No files found matching criteria.${NC}"
        return 1
    fi
    
    # Apply file limit if specified
    if [[ -n "$FILE_LIMIT" && "$FILE_LIMIT" -gt 0 ]]; then
        if [[ $file_count -gt $FILE_LIMIT ]]; then
            echo -e "${YELLOW}Warning: Found $file_count files, limiting to $FILE_LIMIT${NC}"
            files=$(echo "$files" | head -n "$FILE_LIMIT")
            file_count="$FILE_LIMIT"
        fi
    fi
    
    # Show warning for large codebases
    if [[ $file_count -gt 100 ]]; then
        echo -e "${YELLOW}⚠️  Large codebase detected: $file_count files${NC}"
        echo -e "${YELLOW}This analysis may take a long time. Consider using --limit to process fewer files.${NC}"
        read -p "Continue with all $file_count files? [y/N]: " confirm
        if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
            echo -e "${CYAN}Analysis cancelled. Use --limit option to analyze fewer files.${NC}"
            return 1
        fi
    fi
    
    echo -e "${GREEN}Found $file_count files to analyze${NC}"
    echo
    
    local success_count=0
    local total_count=0
    
    while IFS= read -r file; do
        [[ -z "$file" ]] && continue
        
        ((total_count++))
        echo -e "${PURPLE}[$total_count/$file_count] Processing: $(basename "$file")${NC}"
        
        if analyze_file "$file" "$analysis_type"; then
            ((success_count++))
        fi
        
        echo
        echo "---"
        echo
    done <<< "$files"
    
    echo -e "${GREEN}Analysis complete: $success_count/$total_count files processed successfully${NC}"
}

interactive_mode() {
    print_header
    
    echo -e "${CYAN}Welcome to Interactive Code Analysis${NC}"
    echo
    
    # Model selection
    echo -e "${YELLOW}Available models:${NC}"
    list_available_models
    
    read -p "Select model [$MODEL]: " user_model
    [[ -n "$user_model" ]] && MODEL="$user_model"
    
    # Analysis type
    echo
    echo -e "${YELLOW}Analysis types:${NC}"
    echo "1) errors - Find bugs and syntax issues"
    echo "2) style - Code style and best practices"
    echo "3) security - Security vulnerabilities"
    echo "4) performance - Performance optimization"
    echo "5) cleanup - Stub/unused/zombie code removal"
    echo "6) all - Comprehensive analysis"
    
    read -p "Select analysis type [errors]: " analysis_choice
    case "$analysis_choice" in
        1) ANALYSIS_TYPES="errors" ;;
        2) ANALYSIS_TYPES="style" ;;
        3) ANALYSIS_TYPES="security" ;;
        4) ANALYSIS_TYPES="performance" ;;
        5) ANALYSIS_TYPES="cleanup" ;;
        6) ANALYSIS_TYPES="all" ;;
        *) ANALYSIS_TYPES="errors" ;;
    esac
    
    # Target selection
    echo
    echo -e "${YELLOW}What would you like to analyze?${NC}"
    echo "1) Current directory ($(pwd))"
    echo "2) Specific directory"
    echo "3) Single file"
    
    read -p "Select option [1]: " target_choice
    
    case "$target_choice" in
        2)
            read -p "Enter directory path: " target_dir
            [[ -z "$target_dir" ]] && target_dir="$(pwd)"
            ;;
        3)
            read -p "Enter file path: " target_file
            ;;
        *)
            target_dir="$(pwd)"
            ;;
    esac
    
    # Save configuration
    save_config
    
    echo
    echo -e "${GREEN}Starting analysis...${NC}"
    echo
    
    # Run analysis
    if [[ -n "$target_file" ]]; then
        analyze_file "$target_file" "$ANALYSIS_TYPES"
    else
        analyze_directory "$target_dir" "$ANALYSIS_TYPES"
    fi
}

# Main script logic
main() {
    # Load configuration
    load_config
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -m|--model)
                MODEL="$2"
                shift 2
                ;;
            -t|--type)
                ANALYSIS_TYPES="$2"
                shift 2
                ;;
            -e|--extensions)
                FILE_EXTENSIONS="$2"
                shift 2
                ;;
            -r|--recursive)
                RECURSIVE=true
                shift
                ;;
            -f|--file)
                TARGET_FILE="$2"
                shift 2
                ;;
            -c|--config)
                echo "Current Configuration:"
                echo "====================="
                cat "$CONFIG_FILE"
                exit 0
                ;;
            -l|--list-models)
                list_available_models
                exit 0
                ;;
            -i|--interactive)
                INTERACTIVE=true
                shift
                ;;
            -o|--output)
                OUTPUT_FILE="$2"
                shift 2
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -h|--help)
                print_usage
                exit 0
                ;;
            -*)
                echo -e "${RED}Unknown option: $1${NC}"
                print_usage
                exit 1
                ;;
            *)
                TARGET_DIR="$1"
                shift
                ;;
        esac
    done
    
    # Ensure Ollama service is running with correct models path
    if ! ensure_ollama_running; then
        echo -e "${RED}Failed to start Ollama service. Exiting.${NC}"
        exit 1
    fi
    
    # Set OLLAMA_MODELS environment variable
    export OLLAMA_MODELS="$MODELS_PATH"
    
    # Run based on mode
    if [[ "$INTERACTIVE" == "true" ]] || [[ -z "$TARGET_DIR" && -z "$TARGET_FILE" ]]; then
        interactive_mode
    elif [[ -n "$TARGET_FILE" ]]; then
        print_header
        analyze_file "$TARGET_FILE" "$ANALYSIS_TYPES"
    else
        print_header
        analyze_directory "$TARGET_DIR" "$ANALYSIS_TYPES"
    fi
}

# Run main function
main "$@"