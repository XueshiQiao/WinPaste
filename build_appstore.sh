#!/bin/bash
# PastePaw App Store Build Script
# This script builds the universal macOS app and packages it for the App Store.

set -e

# --- Configuration ---
APP_NAME="PastePaw"
# Users must update these with their own identity names from Keychain Access
# e.g. "Apple Distribution: Your Name (TEAM_ID)"
APPLE_DIST_IDENTITY="Apple Distribution: xueshi qiao (584KQTRF3B)"
# e.g. "3rd Party Mac Developer Installer: Your Name (TEAM_ID)"
INSTALLER_DIST_IDENTITY="3rd Party Mac Developer Installer: xueshi qiao (584KQTRF3B)"
APP_PATH="src-tauri/target/universal-apple-darwin/release/bundle/macos/$APP_NAME.app"


echo "🚀 Starting App Store build for $APP_NAME..."

# Kill any running instances
echo "🛑 Ensuring PastePaw is not running..."
pkill -x "$APP_NAME" || true

# Clean previous build artifacts to prevent permission issues
echo "🧹 Cleaning previous build artifacts..."

if [ -d "$APP_PATH" ]; then
  echo "🛑 Removing existing app bundle at $APP_PATH"
  sudo rm -rf "$APP_PATH"
fi

# Check for -bumpversion flag
if [[ "$1" == "-bumpversion" ]]; then
  echo "📈 Bumping patch version..."

  # 1. Bump package.json
  # Use npm version to bump and output new version
  NEW_VERSION=$(npm version patch --no-git-tag-version)
  # Remove 'v' prefix if present
  NEW_VERSION=${NEW_VERSION#v}
  echo "   New version: $NEW_VERSION"

  # 2. Update tauri.conf.json
  # Use a temporary python script for reliable JSON editing
  python3 -c "import json; f='src-tauri/tauri.conf.json'; d=json.load(open(f)); d['version']='$NEW_VERSION'; json.dump(d, open(f,'w'), indent=4)"
  echo "   Updated tauri.conf.json"

  # 3. Update Cargo.toml
  # Simple sed replacement for the version line under [package]
  # We match 'version = "x.y.z"'
  sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml
  echo "   Updated src-tauri/Cargo.toml"

  # 4. Commit the bump (Optional, but good practice for a build script flag)
  # git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
  # git commit -m "chore(release): Bump version to $NEW_VERSION"
fi

# Extract version from package.json
VERSION=$(grep '"version":' package.json | cut -d'"' -f4)
PKG_NAME="${APP_NAME}-${VERSION}.pkg"

# Check for placeholder TEAM_ID in Entitlements.plist
if grep -q "YOUR_TEAM_ID" src-tauri/Entitlements.plist; then
  echo "❌ Error: 'YOUR_TEAM_ID' placeholder found in src-tauri/Entitlements.plist."
  echo "👉 Please replace it with your actual Apple Team ID before building."
  exit 1
fi

# 1. Build Universal Binary using the App Store configuration overlay
echo "📦 Building universal app bundle..."
pnpm tauri build --bundles app --target universal-apple-darwin \
  --config src-tauri/tauri.appstore.conf.json


# 2. Verify signing (Tauri handles basic signing, but we verify it's deep and strict)
echo "🔍 Verifying code signing..."
codesign --verify --deep --strict --verbose=2 "$APP_PATH"

# 3. Build .pkg for App Store submission
echo "🎁 Creating .pkg installer..."
# Note: productbuild requires the Installer certificate
# If certificates are not in Keychain, this step will fail.
xcrun productbuild \
  --sign "$INSTALLER_DIST_IDENTITY" \
  --component "$APP_PATH" /Applications \
  "$PKG_NAME"

echo "✅ Build complete! Output: $PKG_NAME"
echo "👉 You can now validate this package using Transporter or 'xcrun altool --validate-app'."
