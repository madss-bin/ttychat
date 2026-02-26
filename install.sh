#!/bin/bash
set -e

REPO_URL="https://github.com/madss-bin/ttychat.git"
LOGO_FILE="assets/logo.txt"

C_BLUE='\033[38;5;39m'
C_PURPLE='\033[38;5;135m'
C_PINK='\033[38;5;213m'
C_GREEN='\033[38;5;82m'
C_GREY='\033[38;5;240m'
YELLOW='\033[1;33m'
NC='\033[0m'

hide_cursor() { echo -ne "\033[?25l"; }
show_cursor() { echo -ne "\033[?25h"; }
cleanup() {
    show_cursor
}
trap cleanup EXIT

show_logo() {
    local logo="${1:-$LOGO_FILE}"
    [[ ! -f "$logo" ]] && return

    echo
    echo -e "${C_BLUE}"
    cat "$logo"
    echo -e "${NC}"
    echo
}

run_step() {
    local desc="$1"
    shift
    local cmds=("$@")
    local total_steps=${#cmds[@]}
    
    echo -e "${C_PURPLE}:: ${C_BLUE}$desc${NC}"
    echo ""
    echo ""
    echo ""

    for ((i=0; i<total_steps; i++)); do
        local cmd="${cmds[$i]}"
        local step_num=$((i+1))
        local percent=$(( step_num * 100 / total_steps ))
        
        local width=40
        local filled=$(( percent * width / 100 ))
        local empty=$(( width - filled ))
        local bar=$(printf "%0.s━" $(seq 1 $filled))
        local space=$(printf "%0.s━" $(seq 1 $empty))
        
        echo -ne "\033[3A"
        echo -e "\r\033[K${C_GREY}> $cmd${NC}"
        echo -e "\r\033[K${C_GREEN}▕${C_PINK}${bar}${C_GREY}${space}${C_GREEN}▏ ${C_PINK}${percent}%${NC}"

        set +e
        eval "$cmd" 2>&1 | while IFS= read -r line; do
            local trimmed=$(echo "$line" | cut -c 1-70)
            echo -ne "\r\033[K${C_GREY}$trimmed${NC}"
        done
        local exit_code=${PIPESTATUS[0]}
        set -e

        if [ $exit_code -ne 0 ]; then
             echo -e "\n${C_PINK}Command failed: $cmd${NC}"
             exit 1
        fi
        
        echo -ne "\r\033[K"
    done
    echo ""
}

hide_cursor
show_logo

if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    OS=$(uname -s)
fi

echo -e "Detected OS: ${YELLOW}$OS${NC}"

STEPS_DEPS=()
case $OS in
    ubuntu|debian|kali|raspbian)
        STEPS_DEPS+=("sudo apt-get update")
        STEPS_DEPS+=("sudo apt-get install -y build-essential pkg-config libssl-dev curl")
        ;;
    arch|manjaro)
        STEPS_DEPS+=("sudo pacman -S --needed --noconfirm base-devel openssl curl")
        ;;
    fedora|rhel|centos)
        STEPS_DEPS+=("sudo dnf groupinstall -y 'Development Tools'")
        STEPS_DEPS+=("sudo dnf install -y openssl-devel curl")
        ;;
    opensuse*|suse)
        STEPS_DEPS+=("sudo zypper install -t pattern devel_basis")
        STEPS_DEPS+=("sudo zypper install -y libopenssl-devel curl")
        ;;
esac

if [ ${#STEPS_DEPS[@]} -gt 0 ]; then
    run_step "Installing Dependencies" "${STEPS_DEPS[@]}"
fi

if ! command -v cargo &> /dev/null; then
    run_step "Installing Rust" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y" "source $HOME/.cargo/env"
fi

run_step "Building ttychat" "cargo build --release"
run_step "Installing to System" "sudo make install"

echo
echo -e "${C_GREEN}✓ ttychat installed successfully!${NC}"
echo -e "Run ${C_BLUE}ttychat${NC} to start."
