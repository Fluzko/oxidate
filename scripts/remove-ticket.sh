#!/usr/bin/env bash
#
# remove-ticket.sh - Remove a git worktree for a Jira ticket
#
# Usage: ./scripts/remove-ticket.sh TICKET-ID
# Example: ./scripts/remove-ticket.sh MFLP-9

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

# Change to repo root
cd "$REPO_ROOT"

# Check if worktree exists
if [ ! -d "../$TICKET_ID" ]; then
    print_error "Worktree directory '$TICKET_ID' does not exist"
    exit 1
fi

# Check for uncommitted changes
if ! git -C "../$TICKET_ID" diff-index --quiet HEAD -- 2>/dev/null; then
    print_error "Worktree has uncommitted changes. Commit or stash them first."
    git -C "../$TICKET_ID" status --short
    exit 1
fi

# Check if branch is pushed
BRANCH_PUSHED=$(git -C "../$TICKET_ID" rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || echo "")

if [ -z "$BRANCH_PUSHED" ]; then
    print_warn "Branch '$TICKET_ID' is not pushed to remote"
    read -p "Are you sure you want to remove this worktree? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Confirm deletion
print_warn "This will remove the worktree at: $(cd .. && pwd)/$TICKET_ID"
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

# Remove worktree
print_info "Removing worktree..."
git worktree remove "../$TICKET_ID"

# Ask about branch deletion
print_info "Worktree removed successfully"
read -p "Do you also want to delete the branch '$TICKET_ID'? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Delete local branch
    git branch -d "$TICKET_ID" || {
        print_warn "Branch has unmerged commits. Use -D to force delete."
        read -p "Force delete? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git branch -D "$TICKET_ID"
        fi
    }

    # Ask about remote deletion
    if git ls-remote --heads origin "$TICKET_ID" | grep -q "$TICKET_ID"; then
        read -p "Delete remote branch too? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git push origin --delete "$TICKET_ID"
            print_info "Remote branch deleted"
        fi
    fi
fi

print_info "Done!"
