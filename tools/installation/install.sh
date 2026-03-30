#!/bin/bash
# Peel Installation Script (Unix)
set -e

# --- Styles ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# --- Config ---
BASE_URL="https://raw.githubusercontent.com/oopsio/peel/master/built"
INSTALL_DIR="$HOME/.peel/bin"

show_header() {
    echo -e "\n${YELLOW}${BOLD}  P E E L ${NC}"
    echo -e "  ${CYAN}The Peel Programming Language Installer${NC}"
    echo -e "  ---------------------------------------"
}

write_step() {
    echo -e "  ${NC}[+] $1"
}

write_success() {
    echo -e "\n  ${GREEN}${BOLD}[SUCCESS] $1${NC}"
}

write_error() {
    echo -e "\n  ${RED}${BOLD}[ERROR] $1${NC}"
}

detect_os_arch() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "${OS}" in
        Darwin*)
            if [ "${ARCH}" = "arm64" ]; then
                echo "peel-darwin-arm64m"
            else
                write_error "Unsupported macOS architecture: ${ARCH}"
                exit 1
            fi
            ;;
        Linux*)
            if [ "${ARCH}" = "x86_64" ]; then
                echo "peel-linux-x64"
            else
                write_error "Unsupported Linux architecture: ${ARCH}"
                exit 1
            fi
            ;;
        *)
            write_error "Unsupported OS: ${OS}"
            exit 1
            ;;
    esac
}

main() {
    show_header

    # 1. Detect
    BINARY_NAME=$(detect_os_arch)
    DOWNLOAD_URL="${BASE_URL}/${BINARY_NAME}"
    write_step "Detected environment: $(uname -s) (${ARCH})"

    # 2. Prepare Dir
    mkdir -p "${INSTALL_DIR}"
    write_step "Installation directory: ${INSTALL_DIR}"

    # 3. Download
    write_step "Downloading ${BINARY_NAME}..."
    
    if command -v curl >/dev/null 2>&1; then
        curl -L -# -o "${INSTALL_DIR}/peel" "${DOWNLOAD_URL}"
    elif command -v wget >/dev/null 2>&1; then
        wget -q --show-progress -O "${INSTALL_DIR}/peel" "${DOWNLOAD_URL}"
    else
        write_error "Neither curl nor wget found. Please install one to continue."
        exit 1
    fi

    # 4. Permissions
    chmod +x "${INSTALL_DIR}/peel"

    # 5. Path Setup
    SHELL_PROFILE=""
    case "${SHELL}" in
        */zsh*)  SHELL_PROFILE="$HOME/.zshrc" ;;
        */bash*) SHELL_PROFILE="$HOME/.bashrc" ;;
        *)       SHELL_PROFILE="$HOME/.profile" ;;
    esac

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        write_step "Adding ${INSTALL_DIR} to PATH via ${SHELL_PROFILE}..."
        echo -e "\n# Peel Path\nexport PATH=\"\$PATH:${INSTALL_DIR}\"" >> "${SHELL_PROFILE}"
    fi

    write_success "Peel has been successfully installed!"
    echo -e "  Binary: ${INSTALL_DIR}/peel"
    echo -e "  Version: $(${INSTALL_DIR}/peel --version)"
    echo -e "\n  ${YELLOW}Restart your shell or run 'source ${SHELL_PROFILE}' to start using 'peel'.${NC}"
}

main
