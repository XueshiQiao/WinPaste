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

echo "🚀 Starting App Store build for $APP_NAME..."

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

APP_PATH="src-tauri/target/universal-apple-darwin/release/bundle/macos/$APP_NAME.app"

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
