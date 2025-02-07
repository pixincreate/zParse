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
ZPARSE_VERSION="version = \"$VERSION\""

# Update version in workspace Cargo.toml
sed -i.bak "s/^version = .*/$ZPARSE_VERSION/" Cargo.toml

# Commit changes
git add Cargo.toml
git commit -m "chore: bump version to $VERSION"

# Create and push tag
git tag -a "v$VERSION" -m "zParse v$VERSION"
git push origin main "v$VERSION"
