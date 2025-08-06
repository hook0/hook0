#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build"

# Colors for output (only if terminal supports it)
if [[ -t 1 ]] && command -v tput >/dev/null 2>&1 && tput colors >/dev/null 2>&1 && [[ $(tput colors) -ge 8 ]]; then
    GREEN='\033[0;32m'
    BLUE='\033[0;34m'
    YELLOW='\033[1;33m'
    RED='\033[0;31m'
    CYAN='\033[0;36m'
    NC='\033[0m' # No Color
else
    GREEN=''
    BLUE=''
    YELLOW=''
    RED=''
    CYAN=''
    NC=''
fi

print_usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  install   Install dependencies with npm"
    echo "  build     Build documentation (production)"
    echo "  watch     Start development server with watch mode (http://localhost:3000)"
    echo "  deploy    Deploy to GitHub Pages"
    echo "  clean     Clean build directories"
    echo "  help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 install    # First time setup"
    echo "  $0 build      # Build documentation"
    echo "  $0 watch      # Start dev server"
    echo "  $0 deploy     # Deploy to GitHub Pages"
}

check_node() {
    if ! command -v node &> /dev/null; then
        echo -e "${RED}Error: Node.js is not installed${NC}" >&2
        echo -e "${YELLOW}Please install Node.js 18+ first:${NC}" >&2
        echo "  https://nodejs.org/" >&2
        exit 1
    fi
    
    # Check Node version
    NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
    if [ "$NODE_VERSION" -lt 18 ]; then
        echo -e "${RED}Error: Node.js 18+ is required (current: $(node -v))${NC}" >&2
        exit 1
    fi
}

check_dependencies() {
    if [ ! -d "$PROJECT_ROOT/node_modules" ]; then
        echo -e "${RED}Error: Dependencies not installed${NC}" >&2
        echo -e "${YELLOW}Please run '$0 install' first${NC}" >&2
        exit 1
    fi
}

install_dependencies() {
    check_node
    
    echo -e "${BLUE}✅ doc:install - Installing Docusaurus dependencies...${NC}"
    cd "$PROJECT_ROOT"
    
    # Check if npm is available
    if command -v npm &> /dev/null; then
        echo -e "${CYAN}Using npm to install dependencies...${NC}"
        npm install
    else
        echo -e "${RED}Error: npm is not installed${NC}" >&2
        exit 1
    fi
    
    echo -e "${GREEN}✓ Dependencies installed successfully!${NC}"
}

build_docs() {
    check_dependencies
    
    echo -e "${BLUE}✅ doc:build - Building documentation with Docusaurus...${NC}"
    cd "$PROJECT_ROOT"
    
    # Show warnings for missing files
    echo -e "${CYAN}Building production documentation...${NC}"
    
    # Run Docusaurus build
    if npm run build 2>&1 | tee /tmp/docusaurus-build.log; then
        echo -e "${GREEN}✓ Documentation built successfully!${NC}"
    else
        echo -e "${YELLOW}⚠ Build completed with warnings${NC}"
        echo -e "${YELLOW}Check the output above for missing files or broken links${NC}"
    fi
    
    # Check for common issues
    if grep -q "WARNING\|ERROR" /tmp/docusaurus-build.log 2>/dev/null; then
        echo ""
        echo -e "${YELLOW}Issues detected during build:${NC}"
        grep "WARNING\|ERROR" /tmp/docusaurus-build.log | head -10
    fi
    
    echo -e "${GREEN}✓ Generated files are in: $BUILD_DIR${NC}"
}

watch_docs() {
    check_dependencies
    
    echo -e "${BLUE}✅ doc:watch - Starting Docusaurus development server...${NC}"
    echo -e "${CYAN}Documentation will be available at: http://localhost:3000${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
    echo ""
    cd "$PROJECT_ROOT"
    
    # Handle Ctrl+C gracefully
    trap 'echo -e "\n${BLUE}Shutting down development server...${NC}"; exit 0' INT
    
    # Run Docusaurus dev server
    npm run start
}

deploy_docs() {
    check_dependencies
    
    echo -e "${BLUE}✅ doc:deploy - Deploying to GitHub Pages...${NC}"
    cd "$PROJECT_ROOT"
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${RED}Error: Not in a git repository${NC}" >&2
        exit 1
    fi
    
    # Build documentation first
    echo -e "${CYAN}Building documentation for deployment...${NC}"
    npm run build
    
    # Deploy using Docusaurus deploy command
    echo -e "${CYAN}Deploying to GitHub Pages...${NC}"
    
    # Set GIT_USER for deployment
    export GIT_USER=$(git config user.name)
    
    # Deploy
    npm run deploy
    
    echo -e "${GREEN}✓ Documentation deployed to GitHub Pages!${NC}"
    echo -e "${YELLOW}It may take a few minutes for changes to appear at your GitHub Pages URL${NC}"
    
    # Get the repository URL for GitHub Pages
    REPO_URL=$(git config --get remote.origin.url)
    if [[ $REPO_URL == git@github.com:* ]]; then
        # SSH URL format
        REPO_PATH=${REPO_URL#git@github.com:}
        REPO_PATH=${REPO_PATH%.git}
    elif [[ $REPO_URL == https://github.com/* ]]; then
        # HTTPS URL format
        REPO_PATH=${REPO_URL#https://github.com/}
        REPO_PATH=${REPO_PATH%.git}
    fi
    
    if [ ! -z "$REPO_PATH" ]; then
        USER=$(echo $REPO_PATH | cut -d'/' -f1)
        REPO=$(echo $REPO_PATH | cut -d'/' -f2)
        echo -e "${CYAN}Documentation will be available at: https://$USER.github.io/$REPO/${NC}"
    fi
}

clean_docs() {
    echo -e "${BLUE}✅ doc:clean - Cleaning build directories...${NC}"
    
    # Clean build directory
    if [ -d "$BUILD_DIR" ]; then
        echo -e "${CYAN}Removing $BUILD_DIR...${NC}"
        rm -rf "$BUILD_DIR"
    fi
    
    # Clean .docusaurus directory if it exists
    if [ -d "$PROJECT_ROOT/.docusaurus" ]; then
        echo -e "${CYAN}Removing .docusaurus directory...${NC}"
        rm -rf "$PROJECT_ROOT/.docusaurus"
    fi
    
    # Clean node_modules if requested
    if [ "$1" == "--all" ]; then
        if [ -d "$PROJECT_ROOT/node_modules" ]; then
            echo -e "${CYAN}Removing node_modules...${NC}"
            rm -rf "$PROJECT_ROOT/node_modules"
        fi
    fi
    
    echo -e "${GREEN}✓ Build directories cleaned!${NC}"
}

serve_docs() {
    check_dependencies
    
    echo -e "${BLUE}✅ doc:serve - Serving built documentation...${NC}"
    cd "$PROJECT_ROOT"
    
    if [ ! -d "$BUILD_DIR" ]; then
        echo -e "${YELLOW}Build directory not found. Building documentation first...${NC}"
        npm run build
    fi
    
    echo -e "${CYAN}Serving documentation at: http://localhost:3000${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
    
    # Handle Ctrl+C gracefully
    trap 'echo -e "\n${BLUE}Shutting down server...${NC}"; exit 0' INT
    
    npm run serve
}

main() {
    case "${1:-help}" in
        install)
            install_dependencies
            ;;
        build)
            build_docs
            ;;
        watch|start|dev)
            watch_docs
            ;;
        serve)
            serve_docs
            ;;
        deploy)
            deploy_docs
            ;;
        clean)
            clean_docs "$2"
            ;;
        help|--help|-h)
            print_usage
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown command '$1'${NC}" >&2
            echo ""
            print_usage
            exit 1
            ;;
    esac
}

main "$@"