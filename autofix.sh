#!/bin/bash

# Ollama Auto-Fix - Command Line Version
# Automatically fixes code issues using AI

set -e

MODELS_PATH="/run/media/garuda/73cf9511-0af0-4ac4-9d83-ee21eb17ff5d/models"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

print_usage() {
    cat << EOF
Ollama Auto-Fix - Automatically fix code issues

Usage: $0 [OPTIONS] TARGET

OPTIONS:
    -m, --model MODEL       AI model to use (default: granite-code:latest)
    -t, --type TYPE         Fix type: errors, style, cleanup, all (default: cleanup)
    -l, --limit LIMIT       Maximum files to process (default: unlimited)
    -b, --backup           Create backups (default: true)
    --dry-run              Show what would be fixed without making changes
    -h, --help             Show this help

EXAMPLES:
    $0 src/main.rs                    # Fix single file
    $0 /path/to/project               # Fix entire project
    $0 --dry-run src/                 # Preview fixes without applying
    $0 -m deepseek-coder-v2:latest .  # Use specific model
    
WARNING: This tool modifies your files! Always use version control.
EOF
}

autofix_file() {
    local file="$1"
    local model="$2"
    local fix_type="$3"
    local dry_run="$4"
    
    echo -e "${YELLOW}🔍 Analyzing: $(basename "$file")${NC}"
    
    # Skip large files
    local size=$(wc -c < "$file" 2>/dev/null || echo 0)
    if [[ $size -gt 10000 ]]; then
        echo -e "${YELLOW}⚠️  Skipping large file (>10KB): $(basename "$file")${NC}"
        return 0
    fi
    
    # Read file content
    local content
    if ! content=$(cat "$file" 2>/dev/null); then
        echo -e "${RED}❌ Error reading file: $(basename "$file")${NC}"
        return 1
    fi
    
    # Get file extension and language
    local ext="${file##*.}"
    local language=""
    case "$ext" in
        rs) language="rust" ;;
        ts|tsx) language="typescript" ;;
        js|jsx) language="javascript" ;;
        py) language="python" ;;
        go) language="go" ;;
        java) language="java" ;;
        cpp|cc|cxx) language="cpp" ;;
        c) language="c" ;;
        *) language="text" ;;
    esac
    
    # Create fix prompt
    local prompt="Fix the issues in this $language code:

\`\`\`$language
$content
\`\`\`

Please:
1. Fix syntax errors and bugs
2. Remove unused imports and variables  
3. Remove commented-out code
4. Improve code style and formatting
5. Optimize performance where possible

Return ONLY the fixed code in \`\`\`$language code blocks. No explanations."
    
    echo -e "${CYAN}🤖 Running AI analysis with $model...${NC}"
    
    # Run Ollama analysis
    export OLLAMA_MODELS="$MODELS_PATH"
    export CUDA_VISIBLE_DEVICES=0
    
    local response
    if ! response=$(timeout 120 ollama run "$model" "$prompt" 2>/dev/null); then
        echo -e "${RED}❌ AI analysis failed for $(basename "$file")${NC}"
        return 1
    fi
    
    # Extract fixed code from response
    local fixed_code
    fixed_code=$(echo "$response" | sed -n "/\`\`\`$language/,/\`\`\`/p" | sed '1d;$d' || echo "")
    
    if [[ -z "$fixed_code" ]]; then
        echo -e "${BLUE}ℹ️  No fixes suggested for $(basename "$file")${NC}"
        return 0
    fi
    
    # Check if there are actual changes
    if [[ "$fixed_code" == "$content" ]]; then
        echo -e "${BLUE}ℹ️  No changes needed for $(basename "$file")${NC}"
        return 0
    fi
    
    if [[ "$dry_run" == "true" ]]; then
        echo -e "${GREEN}✨ Would fix: $(basename "$file")${NC}"
        echo -e "${CYAN}Preview of changes:${NC}"
        echo "--- Original"
        echo "+++ Fixed"
        diff -u <(echo "$content") <(echo "$fixed_code") || true
        echo ""
        return 0
    fi
    
    # Create backup
    local backup_file="${file}.backup"
    if ! cp "$file" "$backup_file" 2>/dev/null; then
        echo -e "${RED}❌ Failed to create backup for $(basename "$file")${NC}"
        return 1
    fi
    
    # Write fixed code
    if echo "$fixed_code" > "$file"; then
        echo -e "${GREEN}✅ Fixed: $(basename "$file") (backup: $(basename "$backup_file"))${NC}"
        return 0
    else
        echo -e "${RED}❌ Failed to write fixed code to $(basename "$file")${NC}"
        return 1
    fi
}

main() {
    local target=""
    local model="granite-code:latest"
    local fix_type="cleanup"
    local dry_run="false"
    local file_limit="0"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -m|--model)
                model="$2"
                shift 2
                ;;
            -t|--type)
                fix_type="$2"
                shift 2
                ;;
            -l|--limit)
                file_limit="$2"
                shift 2
                ;;
            --dry-run)
                dry_run="true"
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
                target="$1"
                shift
                ;;
        esac
    done
    
    if [[ -z "$target" ]]; then
        echo -e "${RED}Error: No target specified${NC}"
        print_usage
        exit 1
    fi
    
    if [[ ! -e "$target" ]]; then
        echo -e "${RED}Error: Target '$target' does not exist${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}🔧 Ollama Auto-Fix${NC}"
    echo -e "${BLUE}=================${NC}"
    echo -e "${CYAN}Target: $target${NC}"
    echo -e "${CYAN}Model: $model${NC}"
    echo -e "${CYAN}Type: $fix_type${NC}"
    echo -e "${CYAN}Mode: $([ "$dry_run" = "true" ] && echo "DRY RUN" || echo "LIVE FIXES")${NC}"
    echo ""
    
    # Ensure Ollama is running
    export OLLAMA_MODELS="$MODELS_PATH"
    if ! pgrep -f "ollama serve" > /dev/null; then
        echo -e "${YELLOW}Starting Ollama service...${NC}"
        OLLAMA_MODELS="$MODELS_PATH" ollama serve > /dev/null 2>&1 &
        sleep 3
    fi
    
    local files_to_fix=()
    local fixed_count=0
    local total_count=0
    
    # Collect files to process
    if [[ -f "$target" ]]; then
        files_to_fix=("$target")
    else
        # Find code files, excluding common build/dependency directories
        local count=0
        while IFS= read -r -d '' file; do
            # Skip very large files (>50KB)
            local size=$(wc -c < "$file" 2>/dev/null || echo 0)
            if [[ $size -lt 50000 ]]; then
                files_to_fix+=("$file")
                ((count++))
                # Apply file limit if set
                if [[ $file_limit -gt 0 && $count -ge $file_limit ]]; then
                    break
                fi
            fi
        done < <(find "$target" -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.py" -o -name "*.go" -o -name "*.java" -o -name "*.cpp" -o -name "*.c" -o -name "*.h" \) -not -path "*/node_modules/*" -not -path "*/target/*" -not -path "*/build/*" -not -path "*/dist/*" -not -path "*/.git/*" -not -path "*/__pycache__/*" -print0)
    fi
    
    total_count=${#files_to_fix[@]}
    
    if [[ $total_count -eq 0 ]]; then
        echo -e "${YELLOW}No code files found to process${NC}"
        exit 0
    fi
    
    echo -e "${GREEN}Found $total_count files to process${NC}"
    echo ""
    
    # Process each file
    for file in "${files_to_fix[@]}"; do
        if autofix_file "$file" "$model" "$fix_type" "$dry_run"; then
            ((fixed_count++)) || true
        fi
        echo ""
    done
    
    echo -e "${BLUE}===================${NC}"
    if [[ "$dry_run" == "true" ]]; then
        echo -e "${GREEN}🔍 Dry run complete: Would fix $fixed_count/$total_count files${NC}"
        echo -e "${CYAN}Run without --dry-run to apply fixes${NC}"
    else
        echo -e "${GREEN}🎉 Auto-fix complete: Fixed $fixed_count/$total_count files${NC}"
        echo -e "${CYAN}💾 Backup files created with .backup extension${NC}"
        echo -e "${YELLOW}⚠️  Review changes and test thoroughly before committing${NC}"
    fi
}

main "$@"