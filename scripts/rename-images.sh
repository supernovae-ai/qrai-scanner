#!/usr/bin/env bash
#
# rename-images.sh - Rename test images with timing and status information
#
# Author: Thibaut @ SuperNovae Studio
# Description: Runs QR validation benchmark and renames images with performance data
# Usage: ./scripts/rename-images.sh [--dry-run] [--help]
# Exit codes: 0=success, 1=general error, 2=missing dependencies, 3=no images found

set -Eeuo pipefail

# Script metadata
readonly SCRIPT_NAME="$(basename "${BASH_SOURCE[0]}")"
readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly PROJECT_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"
readonly VERSION="1.0.0"

# Configuration
readonly TEST_DIR="${PROJECT_ROOT}/test-images"
readonly CLI_BINARY="${PROJECT_ROOT}/target/release/qraisc"
readonly BACKUP_DIR="${TEST_DIR}/.backup-$(date +%Y%m%d-%H%M%S)"
readonly MAPPING_FILE="${TEST_DIR}/README.md"
readonly TEMP_RESULTS="${TEST_DIR}/.rename-results.tmp"

# Performance tiers (in milliseconds)
readonly TIER_FAST=100
readonly TIER_GOOD=200
readonly TIER_MEDIUM=500
readonly TIER_SLOW=1000

# Color codes (use sparingly in production logs)
readonly C_RESET='\033[0m'
readonly C_BOLD='\033[1m'
readonly C_DIM='\033[2m'
readonly C_RED='\033[31m'
readonly C_GREEN='\033[32m'
readonly C_YELLOW='\033[33m'
readonly C_BLUE='\033[34m'
readonly C_CYAN='\033[36m'

# Global state
declare -i DRY_RUN=0
declare -i VERBOSE=0
declare -i PROCESSED=0
declare -i RENAMED=0
declare -i FAILED=0

# Trap handler for cleanup
cleanup() {
  local exit_code=$?
  if [[ -f "$TEMP_RESULTS" ]]; then
    rm -f -- "$TEMP_RESULTS"
  fi
  if [[ $exit_code -ne 0 ]]; then
    log_error "Script failed with exit code $exit_code"
  fi
}

trap cleanup EXIT

# Logging functions
log_info() {
  printf "${C_BLUE}[INFO]${C_RESET} %s\n" "$*" >&2
}

log_success() {
  printf "${C_GREEN}[OK]${C_RESET} %s\n" "$*" >&2
}

log_warn() {
  printf "${C_YELLOW}[WARN]${C_RESET} %s\n" "$*" >&2
}

log_error() {
  printf "${C_RED}[ERROR]${C_RESET} %s\n" "$*" >&2
}

log_debug() {
  if [[ $VERBOSE -eq 1 ]]; then
    printf "${C_DIM}[DEBUG]${C_RESET} %s\n" "$*" >&2
  fi
}

# Usage and help
usage() {
  cat <<EOF
${C_BOLD}${SCRIPT_NAME}${C_RESET} - Rename test images with performance data

${C_BOLD}USAGE:${C_RESET}
  $SCRIPT_NAME [OPTIONS]

${C_BOLD}OPTIONS:${C_RESET}
  -n, --dry-run     Show what would be done without making changes
  -v, --verbose     Enable verbose output
  -h, --help        Show this help message
  --version         Show version information

${C_BOLD}DESCRIPTION:${C_RESET}
  Runs QR validation benchmarks and renames test images with timing and status:

  Format: {status}_{time}ms_{shortid}.png
  Example: OK_445ms_0287c41e.png
          FAIL_1833ms_10cc058f.png

  Creates backup of original names before renaming.
  Generates performance mapping in $MAPPING_FILE

${C_BOLD}EXAMPLES:${C_RESET}
  $SCRIPT_NAME                # Rename all images
  $SCRIPT_NAME --dry-run      # Preview changes without renaming
  $SCRIPT_NAME --verbose      # Show detailed progress

${C_BOLD}EXIT CODES:${C_RESET}
  0  Success
  1  General error
  2  Missing dependencies
  3  No images found

${C_BOLD}AUTHOR:${C_RESET}
  Thibaut @ SuperNovae Studio

EOF
}

# Parse command-line arguments
parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      -n|--dry-run)
        DRY_RUN=1
        log_info "Dry-run mode enabled"
        shift
        ;;
      -v|--verbose)
        VERBOSE=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      --version)
        echo "$SCRIPT_NAME version $VERSION"
        exit 0
        ;;
      *)
        log_error "Unknown option: $1"
        usage
        exit 1
        ;;
    esac
  done
}

# Validate prerequisites
check_dependencies() {
  local -i missing=0

  # Check for required commands
  local -a required_cmds=(
    "realpath:coreutils"
    "date:coreutils"
  )

  for cmd_spec in "${required_cmds[@]}"; do
    local cmd="${cmd_spec%%:*}"
    local pkg="${cmd_spec##*:}"
    if ! command -v "$cmd" &>/dev/null; then
      log_error "Missing required command: $cmd (install $pkg)"
      missing=1
    fi
  done

  # Check for CLI binary
  if [[ ! -x "$CLI_BINARY" ]]; then
    log_error "CLI binary not found or not executable: $CLI_BINARY"
    log_error "Run: cargo build -p qraisc-cli --release"
    missing=1
  fi

  # Check for test directory
  if [[ ! -d "$TEST_DIR" ]]; then
    log_error "Test directory not found: $TEST_DIR"
    missing=1
  fi

  if [[ $missing -eq 1 ]]; then
    exit 2
  fi
}

# Extract short ID from filename
extract_short_id() {
  local filename="$1"
  # Extract UUID from qrcode-ai-{uuid}.png format
  # Get first 8 chars of UUID
  if [[ "$filename" =~ qrcode-ai-([0-9a-f]{8}) ]]; then
    echo "${BASH_REMATCH[1]}"
  else
    # Fallback: use first 8 chars of filename (without extension)
    local basename="${filename%.png}"
    echo "${basename:0:8}"
  fi
}

# Run validation and extract timing/status
validate_image() {
  local image_path="$1"
  local -i start_ms
  local -i end_ms
  local -i elapsed_ms
  local output
  local status
  local score=""

  # Get millisecond timestamp (portable across macOS/Linux)
  if [[ "$(uname -s)" == "Darwin" ]]; then
    start_ms=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time() * 1000')
  else
    start_ms=$(date +%s%3N)
  fi

  # Run validation with JSON output for reliable parsing
  if output=$("$CLI_BINARY" --json "$image_path" 2>&1); then
    status="OK"
    # Extract score from JSON (handle both ValidationResult and DecodeResult)
    if command -v jq &>/dev/null; then
      score=$(echo "$output" | jq -r '.score // "N/A"' 2>/dev/null || echo "N/A")
    else
      # Fallback: simple grep for score field
      score=$(echo "$output" | grep -oE '"score"[[:space:]]*:[[:space:]]*[0-9]+' | grep -oE '[0-9]+' || echo "N/A")
    fi
  else
    status="FAIL"
    score="0"
  fi

  # Get end timestamp
  if [[ "$(uname -s)" == "Darwin" ]]; then
    end_ms=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time() * 1000')
  else
    end_ms=$(date +%s%3N)
  fi

  elapsed_ms=$((end_ms - start_ms))

  # Output format: status|time_ms|score
  echo "${status}|${elapsed_ms}|${score}"
}

# Create backup of original filenames
create_backup() {
  log_info "Creating backup directory: $BACKUP_DIR"

  if [[ $DRY_RUN -eq 1 ]]; then
    log_debug "[DRY-RUN] Would create: $BACKUP_DIR"
    return 0
  fi

  mkdir -p -- "$BACKUP_DIR"

  # Create manifest of original names
  local manifest="${BACKUP_DIR}/original-names.txt"
  find "$TEST_DIR" -maxdepth 1 -name "*.png" -type f -print0 | \
    while IFS= read -r -d '' file; do
      basename -- "$file"
    done | sort > "$manifest"

  log_success "Backup created with $(wc -l < "$manifest") files listed"
}

# Process all images and collect results
process_images() {
  local -i total=0
  local -a image_files

  log_info "Scanning for test images in: $TEST_DIR"

  # Build array of image files (safe, handles spaces)
  readarray -d '' image_files < <(
    find "$TEST_DIR" -maxdepth 1 -name "qrcode-ai-*.png" -type f -print0 | sort -z
  )

  total=${#image_files[@]}

  if [[ $total -eq 0 ]]; then
    log_error "No test images found in $TEST_DIR"
    exit 3
  fi

  log_info "Found $total test images"
  log_info "Running validations (this may take a while)..."

  # Clear temp results file
  : > "$TEMP_RESULTS"

  local -i count=0
  for image_path in "${image_files[@]}"; do
    count=$((count + 1))
    local filename
    filename="$(basename -- "$image_path")"

    printf "${C_DIM}[%3d/%3d]${C_RESET} Processing: %s..." "$count" "$total" "$filename" >&2

    # Run validation and capture results
    local result
    result=$(validate_image "$image_path")

    local status="${result%%|*}"
    local temp="${result#*|}"
    local time_ms="${temp%%|*}"
    local score="${temp##*|}"
    local short_id
    short_id=$(extract_short_id "$filename")

    # Save to temp file: original_path|status|time_ms|score|short_id
    printf "%s|%s|%s|%s|%s\n" "$image_path" "$status" "$time_ms" "$score" "$short_id" >> "$TEMP_RESULTS"

    if [[ "$status" == "OK" ]]; then
      printf " ${C_GREEN}OK${C_RESET} (%sms, score=%s)\n" "$time_ms" "$score" >&2
    else
      printf " ${C_RED}FAIL${C_RESET} (%sms)\n" "$time_ms" >&2
      FAILED=$((FAILED + 1))
    fi

    PROCESSED=$((PROCESSED + 1))
  done

  log_success "Processed $PROCESSED images ($FAILED failed)"
}

# Generate performance tier label
get_tier_label() {
  local -i time_ms=$1
  local status=$2

  if [[ "$status" == "FAIL" ]]; then
    echo "failed"
  elif [[ $time_ms -lt $TIER_FAST ]]; then
    echo "fast"
  elif [[ $time_ms -lt $TIER_GOOD ]]; then
    echo "good"
  elif [[ $time_ms -lt $TIER_MEDIUM ]]; then
    echo "medium"
  elif [[ $time_ms -lt $TIER_SLOW ]]; then
    echo "slow"
  else
    echo "very-slow"
  fi
}

# Rename images based on results
rename_images() {
  log_info "Renaming images based on validation results..."

  while IFS='|' read -r original_path status time_ms score short_id; do
    local original_name
    original_name="$(basename -- "$original_path")"
    local new_name="${status}_${time_ms}ms_${short_id}.png"
    local new_path="${TEST_DIR}/${new_name}"

    if [[ "$original_name" == "$new_name" ]]; then
      log_debug "Already renamed: $new_name"
      continue
    fi

    if [[ $DRY_RUN -eq 1 ]]; then
      printf "${C_DIM}[DRY-RUN]${C_RESET} %s -> %s\n" "$original_name" "$new_name" >&2
    else
      if mv -- "$original_path" "$new_path"; then
        log_debug "Renamed: $original_name -> $new_name"
        RENAMED=$((RENAMED + 1))
      else
        log_error "Failed to rename: $original_name"
      fi
    fi
  done < "$TEMP_RESULTS"

  if [[ $DRY_RUN -eq 0 ]]; then
    log_success "Renamed $RENAMED images"
  else
    log_info "Would rename $RENAMED images"
  fi
}

# Generate mapping documentation
generate_mapping() {
  log_info "Generating performance mapping: $MAPPING_FILE"

  local temp_md="${MAPPING_FILE}.tmp"

  # Header
  cat > "$temp_md" <<'EOF'
# QR Code Test Images - Performance Mapping

Auto-generated performance analysis of test QR codes.

## Summary Statistics

EOF

  # Calculate statistics
  local -i total
  local -i ok_count=0
  local -i fail_count=0
  local -i total_time=0
  local -i min_time=999999
  local -i max_time=0

  total=$(wc -l < "$TEMP_RESULTS")

  while IFS='|' read -r _ status time_ms score _; do
    if [[ "$status" == "OK" ]]; then
      ok_count=$((ok_count + 1))
    else
      fail_count=$((fail_count + 1))
    fi
    total_time=$((total_time + time_ms))
    if [[ $time_ms -lt $min_time ]]; then
      min_time=$time_ms
    fi
    if [[ $time_ms -gt $max_time ]]; then
      max_time=$time_ms
    fi
  done < "$TEMP_RESULTS"

  local -i avg_time=$((total_time / total))

  cat >> "$temp_md" <<EOF
- **Total Images**: $total
- **Successful**: $ok_count
- **Failed**: $fail_count
- **Average Time**: ${avg_time}ms
- **Min Time**: ${min_time}ms
- **Max Time**: ${max_time}ms

## Performance Tiers

- **Fast**: <${TIER_FAST}ms (excellent)
- **Good**: ${TIER_FAST}-${TIER_GOOD}ms (recommended)
- **Medium**: ${TIER_GOOD}-${TIER_MEDIUM}ms (acceptable)
- **Slow**: ${TIER_SLOW}-${TIER_SLOW}ms (needs optimization)
- **Very Slow**: >${TIER_SLOW}ms (poor performance)
- **Failed**: Validation failed

EOF

  # Group by tier
  declare -A tier_counts
  tier_counts[fast]=0
  tier_counts[good]=0
  tier_counts[medium]=0
  tier_counts[slow]=0
  tier_counts[very-slow]=0
  tier_counts[failed]=0

  while IFS='|' read -r _ status time_ms _ _; do
    local tier
    tier=$(get_tier_label "$time_ms" "$status")
    tier_counts[$tier]=$((tier_counts[$tier] + 1))
  done < "$TEMP_RESULTS"

  # Tier distribution
  cat >> "$temp_md" <<EOF
## Distribution by Tier

| Tier | Count | Percentage |
|------|-------|------------|
| Fast | ${tier_counts[fast]} | $(( tier_counts[fast] * 100 / total ))% |
| Good | ${tier_counts[good]} | $(( tier_counts[good] * 100 / total ))% |
| Medium | ${tier_counts[medium]} | $(( tier_counts[medium] * 100 / total ))% |
| Slow | ${tier_counts[slow]} | $(( tier_counts[slow] * 100 / total ))% |
| Very Slow | ${tier_counts[very-slow]} | $(( tier_counts[very-slow] * 100 / total ))% |
| Failed | ${tier_counts[failed]} | $(( tier_counts[failed] * 100 / total ))% |

EOF

  # Detailed tables grouped by tier
  for tier in fast good medium slow very-slow failed; do
    local tier_title="${tier^}"  # Capitalize first letter
    local -i tier_count=${tier_counts[$tier]}

    if [[ $tier_count -eq 0 ]]; then
      continue
    fi

    cat >> "$temp_md" <<EOF
## ${tier_title} Images (${tier_count})

| Filename | Time (ms) | Score | Status |
|----------|-----------|-------|--------|
EOF

    while IFS='|' read -r original_path status time_ms score short_id; do
      local current_tier
      current_tier=$(get_tier_label "$time_ms" "$status")
      if [[ "$current_tier" == "$tier" ]]; then
        local new_name="${status}_${time_ms}ms_${short_id}.png"
        local original_name
        original_name="$(basename -- "$original_path")"
        printf "| %s | %s | %s | %s |\n" "$new_name" "$time_ms" "$score" "$status" >> "$temp_md"
      fi
    done < <(sort -t'|' -k3 -n "$TEMP_RESULTS")  # Sort by time

    printf "\n" >> "$temp_md"
  done

  # Footer
  cat >> "$temp_md" <<EOF
---

**Generated**: $(date '+%Y-%m-%d %H:%M:%S %Z')
**Script**: $SCRIPT_NAME v$VERSION
**Total Processing Time**: ${total_time}ms
EOF

  if [[ $DRY_RUN -eq 1 ]]; then
    log_info "[DRY-RUN] Would write mapping to: $MAPPING_FILE"
    rm -f -- "$temp_md"
  else
    mv -- "$temp_md" "$MAPPING_FILE"
    log_success "Generated mapping: $MAPPING_FILE"
  fi
}

# Main execution
main() {
  parse_args "$@"

  # Print banner
  cat >&2 <<EOF

${C_BOLD}${C_CYAN}╔═══════════════════════════════════════════════════════╗${C_RESET}
${C_BOLD}${C_CYAN}║${C_RESET}  ${C_BOLD}QRAI Test Image Renamer${C_RESET}                          ${C_BOLD}${C_CYAN}║${C_RESET}
${C_BOLD}${C_CYAN}╚═══════════════════════════════════════════════════════╝${C_RESET}

EOF

  # Validation checks
  check_dependencies

  # Main workflow
  create_backup
  process_images
  generate_mapping
  rename_images

  # Summary
  cat >&2 <<EOF

${C_BOLD}${C_GREEN}╔═══════════════════════════════════════════════════════╗${C_RESET}
${C_BOLD}${C_GREEN}║${C_RESET}  ${C_BOLD}COMPLETED${C_RESET}                                        ${C_BOLD}${C_GREEN}║${C_RESET}
${C_BOLD}${C_GREEN}╚═══════════════════════════════════════════════════════╝${C_RESET}

  ${C_DIM}Processed:${C_RESET}  $PROCESSED images
  ${C_DIM}Renamed:${C_RESET}    $RENAMED images
  ${C_DIM}Failed:${C_RESET}     $FAILED images
  ${C_DIM}Backup:${C_RESET}     $BACKUP_DIR
  ${C_DIM}Mapping:${C_RESET}    $MAPPING_FILE

EOF

  if [[ $DRY_RUN -eq 1 ]]; then
    log_warn "Dry-run mode - no changes were made"
    log_info "Run without --dry-run to apply changes"
  fi
}

# Run main with all arguments
main "$@"
