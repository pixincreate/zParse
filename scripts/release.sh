#!/usr/bin/env bash

# Release Process
# 2. Run release script:
#   ```bash
#   ./scripts/release.sh create_tag 1.0.1
#   ```
# 3. Wait for the tag to be pushed to the repository which will trigger deployment pipeline

set -e

# Check if function and version is provided
if [[ -z "$1" || -z "$2" ]]; then
    echo "Usage: $0 <function_name> <version>"
    echo "Available functions: create_tag, delete_tag"
    echo "Example: $0 create_tag 1.0.0"
    exit 1
fi

FUNCTION="$1"
VERSION="$2"

# Validate version format (semver)
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9\.]+)?$ ]]; then
    echo "Error: Invalid version format '$VERSION'"
    echo "Version must follow semantic versioning: X.Y.Z or X.Y.Z-suffix"
    exit 1
fi

# Validate CHANGELOG.md has been updated
validate_changelog() {
    if ! grep -q "\[Unreleased\]" CHANGELOG.md; then
        echo "Error: CHANGELOG.md doesn't contain an [Unreleased] section"
        echo "Please update CHANGELOG.md before releasing"
        exit 1
    fi

    if grep -q "\[$VERSION\]" CHANGELOG.md; then
        echo "Warning: Version $VERSION already exists in CHANGELOG.md"
        echo "This may cause issues with the automated release process"
        read -r -p "Continue anyway? (y/N): " confirm
        if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
            exit 1
        fi
    fi
}


create_tag() {
    # Ensure we're on the main branch
    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$CURRENT_BRANCH" != "main" ]]; then
        echo "Error: You must be on the main branch to create a release tag"
        exit 1
    fi

    # Ensure working directory is clean
    if ! git diff-index --quiet HEAD --; then
        echo "Error: You have uncommitted changes"
        echo "Please commit or stash them before creating a release"
        exit 1
    fi

    # Validate changelog
    validate_changelog

    # Confirm with user
    echo "This will create and push tag v$VERSION, triggering the release workflow."
    echo "Changelog entries from the [Unreleased] section will be moved to version $VERSION."
    read -p "Continue? (y/N): " confirm
    if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
        echo "Operation cancelled"
        exit 0
    fi

    echo "Creating tag v$VERSION..."
    git tag -a "v$VERSION" -m "zParse v$VERSION"
    git push origin "v$VERSION"
    echo "Tag v$VERSION created and pushed!"
    echo "The release workflow should start automatically. Monitor progress at:"
    echo "https://github.com/pixincreate/zparse/actions"
}

delete_tag() {
    echo "Warning: This will delete tag v$VERSION locally and remotely."
    read -r -p "Continue? (y/N): " confirm
    if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
        echo "Operation cancelled"
        exit 0
    fi

    echo "Deleting tag v$VERSION..."
    git tag -d "v$VERSION" 2>/dev/null || echo "Tag v$VERSION not found locally"
    git push --delete origin "v$VERSION" 2>/dev/null || echo "Tag v$VERSION not found on remote"
    echo "Tag v$VERSION deleted!"
}

# Check if function exists
if [[ "$FUNCTION" != "create_tag" && "$FUNCTION" != "delete_tag" ]]; then
    echo "Error: Unknown function '$FUNCTION'"
    echo "Available functions: create_tag, delete_tag"
    exit 1
fi

eval "$FUNCTION"
