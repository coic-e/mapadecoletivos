#!/bin/bash
# Git submodule management helper script
# This replaces the old makefile commands

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

# Setup: Initialize submodules
setup() {
    print_step "Setting up submodules..."
    git fetch && git pull origin main
    git submodule update --init --recursive
    print_success "Submodules initialized"
}

# Update all submodules to their main branch
update_main() {
    print_step "Updating to main branch..."
    git fetch && git pull origin main
    git submodule update --init --recursive
    git submodule foreach 'git fetch'
    git submodule foreach 'git checkout main || git checkout master'
    git submodule foreach 'git pull origin $(git branch --show-current)'
    print_success "All submodules updated to main/master"
}

# Update all submodules to their develop branch
update_develop() {
    print_step "Updating to develop branch..."
    git fetch && git pull origin main
    git submodule update --init --recursive
    git submodule foreach 'git fetch'
    git submodule foreach 'git checkout develop'
    git submodule foreach 'git pull origin develop'
    print_success "All submodules updated to develop"
}

# Show help
show_help() {
    echo "Usage: ./git-commands.sh [command]"
    echo ""
    echo "Commands:"
    echo "  setup           Initialize and setup all submodules"
    echo "  update-main     Update all submodules to main/master branch"
    echo "  update-develop  Update all submodules to develop branch"
    echo "  help            Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./git-commands.sh setup"
    echo "  ./git-commands.sh update-develop"
}

# Main script logic
case "${1:-help}" in
    setup)
        setup
        ;;
    update-main|main)
        update_main
        ;;
    update-develop|develop)
        update_develop
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
