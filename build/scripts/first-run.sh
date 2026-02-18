#!/bin/bash
# ============================================================================
# Winux OS - First Run Assistant
# Sprint 15-16: Build System and Installer
# ============================================================================
# This script guides new users through initial system setup after installation.
# It runs automatically on first login and helps configure personal preferences.
# ============================================================================

set -e

# Configuration
CONFIG_DIR="$HOME/.config/winux"
SETUP_COMPLETE_FILE="$CONFIG_DIR/setup-complete"
LOG_FILE="$CONFIG_DIR/first-run.log"

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'
BOLD='\033[1m'

# ============================================================================
# Initialization
# ============================================================================

init() {
    # Create config directory
    mkdir -p "$CONFIG_DIR"

    # Start logging
    exec > >(tee -a "$LOG_FILE") 2>&1
    echo "=== Winux First Run - $(date) ===" >> "$LOG_FILE"

    # Check if already completed
    if [[ -f "$SETUP_COMPLETE_FILE" ]]; then
        echo "Setup already completed. Exiting."
        exit 0
    fi
}

# ============================================================================
# UI Functions
# ============================================================================

clear_screen() {
    clear
}

print_header() {
    clear_screen
    echo -e "${MAGENTA}"
    cat << 'EOF'
 __        ___                   ___  ____
 \ \      / (_)_ __  _   ___  __/ _ \/ ___|
  \ \ /\ / /| | '_ \| | | \ \/ / | | \___ \
   \ V  V / | | | | | |_| |>  <| |_| |___) |
    \_/\_/  |_|_| |_|\__,_/_/\_\\___/|____/
EOF
    echo -e "${NC}"
    echo -e "${BOLD}Welcome to Winux OS!${NC}"
    echo -e "${CYAN}Let's set up your new system together.${NC}"
    echo
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo
}

print_step() {
    local step=$1
    local total=$2
    local title=$3
    echo -e "${BLUE}Step ${step}/${total}:${NC} ${BOLD}${title}${NC}"
    echo
}

print_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

prompt_yes_no() {
    local prompt=$1
    local default=${2:-y}
    local response

    if [[ "$default" == "y" ]]; then
        read -p "$prompt [Y/n]: " response
        response=${response:-y}
    else
        read -p "$prompt [y/N]: " response
        response=${response:-n}
    fi

    [[ "$response" =~ ^[Yy] ]]
}

prompt_choice() {
    local prompt=$1
    shift
    local options=("$@")
    local choice

    echo "$prompt"
    for i in "${!options[@]}"; do
        echo "  $((i+1)). ${options[$i]}"
    done

    while true; do
        read -p "Enter your choice [1-${#options[@]}]: " choice
        if [[ "$choice" =~ ^[0-9]+$ ]] && (( choice >= 1 && choice <= ${#options[@]} )); then
            return $((choice - 1))
        fi
        echo "Invalid choice. Please try again."
    done
}

wait_key() {
    read -p "Press Enter to continue..."
}

# ============================================================================
# Step 1: Language and Region
# ============================================================================

step_language() {
    print_header
    print_step 1 7 "Language & Region"

    echo "Your current locale: $(locale | grep LANG | cut -d= -f2)"
    echo

    if prompt_yes_no "Would you like to change your language/locale?"; then
        local languages=("English (US)" "English (UK)" "Spanish" "French" "German" "Portuguese (Brazil)" "Other")
        prompt_choice "Select your preferred language:" "${languages[@]}"
        local lang_choice=$?

        case $lang_choice in
            0) SELECTED_LOCALE="en_US.UTF-8" ;;
            1) SELECTED_LOCALE="en_GB.UTF-8" ;;
            2) SELECTED_LOCALE="es_ES.UTF-8" ;;
            3) SELECTED_LOCALE="fr_FR.UTF-8" ;;
            4) SELECTED_LOCALE="de_DE.UTF-8" ;;
            5) SELECTED_LOCALE="pt_BR.UTF-8" ;;
            6)
                read -p "Enter locale code (e.g., ja_JP.UTF-8): " SELECTED_LOCALE
                ;;
        esac

        # Apply locale
        echo "LANG=$SELECTED_LOCALE" > "$CONFIG_DIR/locale"
        print_success "Language preference saved"
    else
        print_info "Keeping current language settings"
    fi

    echo
    wait_key
}

# ============================================================================
# Step 2: Keyboard Layout
# ============================================================================

step_keyboard() {
    print_header
    print_step 2 7 "Keyboard Layout"

    current_layout=$(setxkbmap -query 2>/dev/null | grep layout | awk '{print $2}' || echo "us")
    echo "Your current keyboard layout: $current_layout"
    echo

    if prompt_yes_no "Would you like to change your keyboard layout?"; then
        local layouts=("US English" "UK English" "Spanish" "French" "German" "Portuguese (Brazil)" "Other")
        prompt_choice "Select your keyboard layout:" "${layouts[@]}"
        local layout_choice=$?

        case $layout_choice in
            0) SELECTED_LAYOUT="us" ;;
            1) SELECTED_LAYOUT="gb" ;;
            2) SELECTED_LAYOUT="es" ;;
            3) SELECTED_LAYOUT="fr" ;;
            4) SELECTED_LAYOUT="de" ;;
            5) SELECTED_LAYOUT="br" ;;
            6)
                read -p "Enter layout code (e.g., jp): " SELECTED_LAYOUT
                ;;
        esac

        # Apply keyboard layout
        setxkbmap "$SELECTED_LAYOUT" 2>/dev/null || true
        echo "$SELECTED_LAYOUT" > "$CONFIG_DIR/keyboard"
        print_success "Keyboard layout changed to: $SELECTED_LAYOUT"
    else
        print_info "Keeping current keyboard layout"
    fi

    echo
    wait_key
}

# ============================================================================
# Step 3: Timezone
# ============================================================================

step_timezone() {
    print_header
    print_step 3 7 "Timezone"

    current_tz=$(timedatectl show --property=Timezone --value 2>/dev/null || echo "UTC")
    echo "Your current timezone: $current_tz"
    echo

    if prompt_yes_no "Would you like to change your timezone?"; then
        local regions=("America" "Europe" "Asia" "Australia" "Pacific" "Other")
        prompt_choice "Select your region:" "${regions[@]}"
        local region_choice=$?

        case $region_choice in
            0) region="America" ;;
            1) region="Europe" ;;
            2) region="Asia" ;;
            3) region="Australia" ;;
            4) region="Pacific" ;;
            5)
                read -p "Enter region (e.g., Africa): " region
                ;;
        esac

        echo
        echo "Available cities in $region:"
        ls /usr/share/zoneinfo/$region 2>/dev/null | head -20 || true
        echo "..."
        echo

        read -p "Enter city name: " city

        if [[ -f "/usr/share/zoneinfo/$region/$city" ]]; then
            sudo timedatectl set-timezone "$region/$city" 2>/dev/null || true
            echo "$region/$city" > "$CONFIG_DIR/timezone"
            print_success "Timezone set to: $region/$city"
        else
            print_warning "Invalid timezone. Keeping current setting."
        fi
    else
        print_info "Keeping current timezone"
    fi

    echo
    wait_key
}

# ============================================================================
# Step 4: Appearance
# ============================================================================

step_appearance() {
    print_header
    print_step 4 7 "Appearance"

    echo "Let's customize how Winux OS looks!"
    echo

    # Theme selection
    local themes=("Winux Dark (Default)" "Winux Light" "Breeze Dark" "Breeze Light")
    prompt_choice "Select your preferred theme:" "${themes[@]}"
    local theme_choice=$?

    case $theme_choice in
        0) SELECTED_THEME="winux-dark" ;;
        1) SELECTED_THEME="winux-light" ;;
        2) SELECTED_THEME="breeze-dark" ;;
        3) SELECTED_THEME="breeze" ;;
    esac

    echo "$SELECTED_THEME" > "$CONFIG_DIR/theme"

    # Apply KDE theme if available
    if command -v lookandfeeltool &>/dev/null; then
        lookandfeeltool -a "org.kde.${SELECTED_THEME}.desktop" 2>/dev/null || true
    fi

    print_success "Theme preference saved: $SELECTED_THEME"

    echo

    # Accent color
    if prompt_yes_no "Would you like to choose an accent color?"; then
        local colors=("Winux Red" "Blue" "Green" "Purple" "Orange" "Teal")
        prompt_choice "Select your accent color:" "${colors[@]}"
        local color_choice=$?

        case $color_choice in
            0) ACCENT_COLOR="#e94560" ;;
            1) ACCENT_COLOR="#3daee9" ;;
            2) ACCENT_COLOR="#27ae60" ;;
            3) ACCENT_COLOR="#9b59b6" ;;
            4) ACCENT_COLOR="#f39c12" ;;
            5) ACCENT_COLOR="#16a085" ;;
        esac

        echo "$ACCENT_COLOR" > "$CONFIG_DIR/accent-color"
        print_success "Accent color saved"
    fi

    echo
    wait_key
}

# ============================================================================
# Step 5: Privacy Settings
# ============================================================================

step_privacy() {
    print_header
    print_step 5 7 "Privacy Settings"

    echo "Winux OS respects your privacy. Let's configure your preferences."
    echo

    # Location services
    if prompt_yes_no "Enable location services? (for weather, timezone auto-detection)" "n"; then
        echo "location=enabled" >> "$CONFIG_DIR/privacy"
        print_info "Location services enabled"
    else
        echo "location=disabled" >> "$CONFIG_DIR/privacy"
        print_info "Location services disabled"
    fi

    echo

    # Crash reporting
    if prompt_yes_no "Send anonymous crash reports to help improve Winux OS?" "n"; then
        echo "crash_reports=enabled" >> "$CONFIG_DIR/privacy"
        print_info "Crash reporting enabled"
    else
        echo "crash_reports=disabled" >> "$CONFIG_DIR/privacy"
        print_info "Crash reporting disabled"
    fi

    echo

    # Usage statistics
    if prompt_yes_no "Share anonymous usage statistics?" "n"; then
        echo "usage_stats=enabled" >> "$CONFIG_DIR/privacy"
        print_info "Usage statistics enabled"
    else
        echo "usage_stats=disabled" >> "$CONFIG_DIR/privacy"
        print_info "Usage statistics disabled"
    fi

    echo
    print_success "Privacy settings saved"

    echo
    wait_key
}

# ============================================================================
# Step 6: Software & Updates
# ============================================================================

step_software() {
    print_header
    print_step 6 7 "Software & Updates"

    echo "Let's set up your software preferences."
    echo

    # Enable Flathub
    if prompt_yes_no "Enable Flathub for additional software?" "y"; then
        if command -v flatpak &>/dev/null; then
            flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo 2>/dev/null || true
            print_success "Flathub enabled"
        else
            print_warning "Flatpak not installed"
        fi
    fi

    echo

    # AUR helper
    if prompt_yes_no "Enable AUR (Arch User Repository) access?" "y"; then
        echo "aur=enabled" >> "$CONFIG_DIR/software"
        print_info "AUR access enabled"
    else
        echo "aur=disabled" >> "$CONFIG_DIR/software"
    fi

    echo

    # Auto updates
    local update_options=("Automatic (recommended)" "Notify only" "Manual")
    prompt_choice "How would you like to handle system updates?" "${update_options[@]}"
    local update_choice=$?

    case $update_choice in
        0)
            echo "updates=automatic" >> "$CONFIG_DIR/software"
            print_info "Automatic updates enabled"
            ;;
        1)
            echo "updates=notify" >> "$CONFIG_DIR/software"
            print_info "Update notifications enabled"
            ;;
        2)
            echo "updates=manual" >> "$CONFIG_DIR/software"
            print_info "Manual updates selected"
            ;;
    esac

    echo

    # Recommended apps
    if prompt_yes_no "Would you like to install some recommended applications?" "y"; then
        echo
        echo "Recommended applications:"
        echo "  - Firefox (Web Browser)"
        echo "  - LibreOffice (Office Suite)"
        echo "  - VLC (Media Player)"
        echo "  - GIMP (Image Editor)"
        echo "  - Kdenlive (Video Editor)"
        echo

        if prompt_yes_no "Install these applications now?" "y"; then
            print_info "Installing recommended applications..."
            # This would normally run: sudo pacman -S --noconfirm firefox libreoffice-fresh vlc gimp kdenlive
            echo "recommended_apps=pending" >> "$CONFIG_DIR/software"
            print_success "Applications will be installed after setup"
        fi
    fi

    echo
    wait_key
}

# ============================================================================
# Step 7: Online Accounts
# ============================================================================

step_accounts() {
    print_header
    print_step 7 7 "Online Accounts"

    echo "Connect your online accounts for a seamless experience."
    echo "(This step is optional)"
    echo

    if prompt_yes_no "Would you like to set up online accounts now?" "n"; then
        # Open KDE account settings
        if command -v kcmshell5 &>/dev/null; then
            print_info "Opening Online Accounts settings..."
            kcmshell5 kcm_kaccounts 2>/dev/null &
        elif command -v systemsettings &>/dev/null; then
            print_info "Opening System Settings..."
            systemsettings 2>/dev/null &
        else
            print_warning "Online accounts configuration not available"
        fi
    else
        print_info "You can set up accounts later in System Settings"
    fi

    echo
    wait_key
}

# ============================================================================
# Completion
# ============================================================================

complete_setup() {
    print_header

    echo -e "${GREEN}${BOLD}"
    cat << 'EOF'
   ____                      _      _       _
  / ___|___  _ __ ___  _ __ | | ___| |_ ___| |
 | |   / _ \| '_ ` _ \| '_ \| |/ _ \ __/ _ \ |
 | |__| (_) | | | | | | |_) | |  __/ ||  __/_|
  \____\___/|_| |_| |_| .__/|_|\___|\__\___(_)
                      |_|
EOF
    echo -e "${NC}"

    echo -e "${BOLD}Your Winux OS setup is complete!${NC}"
    echo
    echo "Here's what we configured:"
    echo

    [[ -f "$CONFIG_DIR/locale" ]] && echo "  - Language: $(cat "$CONFIG_DIR/locale")"
    [[ -f "$CONFIG_DIR/keyboard" ]] && echo "  - Keyboard: $(cat "$CONFIG_DIR/keyboard")"
    [[ -f "$CONFIG_DIR/timezone" ]] && echo "  - Timezone: $(cat "$CONFIG_DIR/timezone")"
    [[ -f "$CONFIG_DIR/theme" ]] && echo "  - Theme: $(cat "$CONFIG_DIR/theme")"

    echo
    echo -e "${CYAN}Getting Started:${NC}"
    echo "  - Press Super key to open the application menu"
    echo "  - Right-click the desktop to customize"
    echo "  - Visit System Settings for more options"
    echo "  - Check out the Winux Welcome app for tips"
    echo
    echo -e "${CYAN}Need Help?${NC}"
    echo "  - Documentation: https://docs.winux.io"
    echo "  - Community: https://forum.winux.io"
    echo "  - Support: https://winux.io/support"
    echo

    # Mark setup as complete
    date > "$SETUP_COMPLETE_FILE"

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo
    echo -e "${GREEN}Enjoy your new Winux OS experience!${NC}"
    echo

    wait_key
}

# ============================================================================
# Main Execution
# ============================================================================

main() {
    init

    # Run all steps
    step_language
    step_keyboard
    step_timezone
    step_appearance
    step_privacy
    step_software
    step_accounts
    complete_setup

    # Open welcome app
    if command -v winux-welcome &>/dev/null; then
        winux-welcome &
    fi
}

# Handle script arguments
case "${1:-}" in
    --skip-to)
        # Allow skipping to specific step for debugging
        shift
        step_name="step_$1"
        if declare -f "$step_name" >/dev/null; then
            init
            $step_name
            complete_setup
        else
            echo "Unknown step: $1"
            exit 1
        fi
        ;;
    --reset)
        # Reset first-run status
        rm -f "$SETUP_COMPLETE_FILE"
        echo "First-run status reset. Run again to start setup."
        ;;
    --help)
        echo "Winux OS First Run Assistant"
        echo
        echo "Usage: $0 [options]"
        echo
        echo "Options:"
        echo "  --help       Show this help message"
        echo "  --reset      Reset first-run status"
        echo "  --skip-to X  Skip to step X (for debugging)"
        ;;
    *)
        main
        ;;
esac

exit 0
