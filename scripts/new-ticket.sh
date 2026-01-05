#!/usr/bin/env bash
#
# new-ticket.sh - Create a new git worktree for a Jira ticket
#
# Usage: ./scripts/new-ticket.sh TICKET-ID
# Example: ./scripts/new-ticket.sh MFLP-9

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory and repo root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Validate arguments
if [ $# -ne 1 ]; then
    print_error "Usage: $0 TICKET-ID"
    print_error "Example: $0 MFLP-9"
    exit 1
fi

TICKET_ID="$1"

# Validate ticket ID format (basic check)
if ! [[ "$TICKET_ID" =~ ^[A-Z]+-[0-9]+$ ]]; then
    print_error "Invalid ticket ID format. Expected format: PROJECT-NUMBER (e.g., MFLP-9)"
    exit 1
fi

# Change to repo root
cd "$REPO_ROOT"

# Check if worktree already exists
if [ -d "../$TICKET_ID" ]; then
    print_error "Worktree directory '$TICKET_ID' already exists"
    exit 1
fi

# Check if branch already exists locally
if git show-ref --verify --quiet "refs/heads/$TICKET_ID"; then
    print_warn "Branch '$TICKET_ID' already exists locally"
    read -p "Do you want to create a worktree from the existing branch? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
    CREATE_BRANCH=false
else
    CREATE_BRANCH=true
fi

# Check if branch exists on remote
if git ls-remote --heads origin "$TICKET_ID" | grep -q "$TICKET_ID"; then
    print_warn "Branch '$TICKET_ID' exists on remote"
    print_info "Creating worktree from remote branch"
    git fetch origin "$TICKET_ID:$TICKET_ID"
    CREATE_BRANCH=false
fi

# Create worktree
print_info "Creating worktree for $TICKET_ID..."

if [ "$CREATE_BRANCH" = true ]; then
    # Create new branch from main
    git worktree add "../$TICKET_ID" -b "$TICKET_ID" main
else
    # Use existing branch
    git worktree add "../$TICKET_ID" "$TICKET_ID"
fi

# Copy .env file if it exists
if [ -f ".env" ]; then
    cp .env "../$TICKET_ID/.env"
    print_info "Copied .env file to worktree"
fi

# Success message
print_info "Worktree created successfully at: $(cd .. && pwd)/$TICKET_ID"
print_info ""
print_info "Next steps:"
print_info "  1. cd ../$TICKET_ID"
print_info "  2. Start working on the ticket"
print_info "  3. Commit and push changes"
print_info ""
print_info "To remove this worktree later:"
print_info "  cd $REPO_ROOT"
print_info "  ./scripts/remove-ticket.sh $TICKET_ID"
