#!/bin/bash
# release.sh - Automates the release process for bmv

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 1. Check if version is provided
if [ -z "$1" ]; then
    echo -e "${RED}Usage: $0 <version>${NC}"
    echo "Example: $0 0.3.0"
    exit 1
fi

VERSION=$1

# 2. Basic validation of version format (x.y.z)
if [[ ! $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Version must be in x.y.z format (e.g., 0.3.0)${NC}"
    exit 1
fi

# 3. Ensure we are in the root of the repo
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: This script must be run from the root of the repository.${NC}"
    exit 1
fi

# 4. Ensure git status is clean
if [[ -n $(git status -s) ]]; then
    echo -e "${RED}Error: Git working directory is not clean. Please commit or stash changes first.${NC}"
    git status -s
    exit 1
fi

# 5. Ensure we are on the main branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "${YELLOW}Warning: You are on branch '$CURRENT_BRANCH', not 'main'.${NC}"
    read -p "Do you want to continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborting."
        exit 1
    fi
fi

echo -e "${GREEN}Preparing release v$VERSION...${NC}"

# 6. Run tests to ensure quality
echo "Running tests..."
if ! ./test.sh; then
    echo -e "${RED}Error: Tests failed. Aborting release.${NC}"
    exit 1
fi

# 7. Update Cargo.toml
echo "Updating Cargo.toml version to $VERSION..."
# Use a cross-platform sed approach
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
else
    sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
fi

# 8. Update Cargo.lock
echo "Updating Cargo.lock..."
cargo check > /dev/null 2>&1

# 9. Commit the version change
echo "Committing version update..."
git add Cargo.toml Cargo.lock
git commit -m "chore: release v$VERSION" -m "Automated version bump to v$VERSION and update of Cargo.lock."

# 10. Create a git tag
echo "Creating git tag v$VERSION..."
git tag -a "v$VERSION" -m "v$VERSION"

echo -e "${GREEN}Release v$VERSION prepared locally.${NC}"

# 11. Push the changes
echo -e "${YELLOW}Pushing changes to origin...${NC}"
git push origin "$CURRENT_BRANCH"
git push origin "v$VERSION"

echo -e "${GREEN}Successfully released and pushed v$VERSION!${NC}"
