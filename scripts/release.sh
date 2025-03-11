#!/usr/bin/env bash

# Release Process
# 1. Update CHANGELOG.md with new version and changes
# 2. Run release script:
#   ```bash
#   ./scripts/release.sh 1.0.1
#   ```
# 3. Wait for GitHub Actions to:
#   - Build binaries for all platforms
#   - Create GitHub release
#   - Upload assets
# 4. Verify the release at: https://github.com/pixincreate/zparse/releases

set -e

# Check if version is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

VERSION="$1"
CRATE_VERSION="version = \"$VERSION\""
DATE=$(date +%Y-%m-%d)

# Check current branch and manage release branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
VERSION_BRANCH="release/$VERSION"

echo "Current branch: $CURRENT_BRANCH"
echo "Target release branch: $VERSION_BRANCH"

# Check if already on the release branch
if [ "$CURRENT_BRANCH" != "$VERSION_BRANCH" ]; then
    echo "Not on the release branch. Checking if release branch exists..."

    # Check if branch exists locally
    if git show-ref --quiet refs/heads/"$VERSION_BRANCH"; then
        echo "Release branch exists locally. Checking out $VERSION_BRANCH..."
        git checkout "$VERSION_BRANCH"
    else
        # Check if branch exists remotely
        if git ls-remote --heads --exit-code origin "$VERSION_BRANCH" > /dev/null 2>&1; then
            echo "Release branch exists in remote. Fetching and checking out $VERSION_BRANCH..."
            git fetch origin "$VERSION_BRANCH":"$VERSION_BRANCH"
            git checkout "$VERSION_BRANCH"
        else
            echo "Release branch does not exist. Creating new branch $VERSION_BRANCH from current branch..."
            git checkout -b "$VERSION_BRANCH"
            echo "Created new branch: $VERSION_BRANCH"
        fi
    fi

    # Verify we're on the correct branch now
    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
    if [ "$CURRENT_BRANCH" != "$VERSION_BRANCH" ]; then
        echo "ERROR: Failed to switch to release branch. Currently on: $CURRENT_BRANCH"
        echo "Please manually resolve branch issues before continuing."
        exit 1
    fi

    echo "Successfully switched to release branch: $CURRENT_BRANCH"
else
    echo "Already on the correct release branch: $CURRENT_BRANCH"
fi

# Update version in workspace Cargo.toml
sed -i.bak "s/^version = .*/$CRATE_VERSION/" Cargo.toml
rm Cargo.toml.bak  # Clean up backup file

# Update CHANGELOG.md
sed -i.bak "s/## \[Unreleased\]/## [Unreleased]\n\n## [$VERSION] - $DATE/" CHANGELOG.md
rm CHANGELOG.md.bak  # Clean up backup file

# Show changes
echo "Changes made:"
echo "------------"
git diff

# Prompt for confirmation
read -p "Do you want to commit these changes and create tag v$VERSION? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    # Commit changes
    git add Cargo.toml CHANGELOG.md
    git commit -m "chore: release version $VERSION"

    # Create and push tag
    git tag -a "v$VERSION" -m "zParse v$VERSION"

    echo "Pushing changes..."
    git push origin main "v$VERSION"

    echo "zParse release v$VERSION prepared and pushed!"
else
    # Revert changes if user doesn't confirm
    git checkout Cargo.toml CHANGELOG.md
    echo "zParse release cancelled and changes reverted"
fi
