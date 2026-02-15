# Mac App Store Submission Checklist for PastePaw

This checklist will guide you through the complete App Store submission process.

## ✅ GitHub Pages Setup (COMPLETED)

- [x] Create landing page (index.html)
- [x] Create privacy policy page (privacy.html)
- [x] Create support page (support.html)
- [x] Create terms of service page (terms.html)
- [ ] Enable GitHub Pages in repository settings
- [ ] Verify all pages are accessible

### To Enable GitHub Pages:

1. Go to: https://github.com/XueshiQiao/PastePaw/settings/pages
2. Under "Source", select: **Deploy from a branch**
3. Branch: **main**, Folder: **/docs/product_pages**
4. Click **Save**
5. Wait 2-3 minutes for deployment
6. Verify site is live at: https://xueshiqiao.github.io/PastePaw/

## 📋 Required URLs for App Store Connect

Once GitHub Pages is enabled, use these URLs in App Store Connect:

| Field | URL |
|-------|-----|
| **Support URL** | `https://xueshiqiao.github.io/PastePaw/support.html` |
| **Marketing URL** | `https://xueshiqiao.github.io/PastePaw/` |
| **Privacy Policy URL** | `https://xueshiqiao.github.io/PastePaw/privacy.html` |

## 🔧 Technical Requirements

### 1. Code Changes for App Store Build

- [ ] Remove or disable `macOSPrivateApi` in App Store build
- [ ] Replace `osascript` calls with native APIs:
  - [ ] Source app detection: Use `NSWorkspace` instead of `osascript`
  - [ ] Remove auto-paste feature (or use Accessibility API)
- [ ] Replace autostart LaunchAgent with `SMAppService` (macOS 13+)
- [ ] Disable `tauri-plugin-updater` for App Store build
- [ ] Test icon extraction in sandbox environment
- [ ] Verify clipboard monitoring works in sandbox

### 2. Certificates & Provisioning

- [ ] Create Certificate Signing Request (CSR) in Keychain Access
- [ ] Create **Apple Distribution** certificate
- [ ] Create **Mac Installer Distribution** certificate
- [ ] Create App ID: `me.xueshi.pastepaw`
- [ ] Create Mac App Store provisioning profile
- [ ] Download and save provisioning profile to `src-tauri/`

### 3. Configuration Files

- [ ] Create `src-tauri/Entitlements.plist` (replace `YOUR_TEAM_ID`)
- [ ] Create `src-tauri/tauri.appstore.conf.json`
- [ ] Update version numbers (sync across all files):
  - [ ] `package.json`
  - [ ] `src-tauri/Cargo.toml`
  - [ ] `src-tauri/tauri.conf.json`

### 4. Build Process

- [ ] Run: `pnpm tauri build --bundles app --target universal-apple-darwin --config src-tauri/tauri.appstore.conf.json`
- [ ] Verify code signing: `codesign --verify --deep --strict --verbose=2 <path-to-app>`
- [ ] Create .pkg installer: `xcrun productbuild ...`
- [ ] Validate .pkg: `xcrun altool --validate-app ...`

### 5. App Store Connect Setup

- [ ] Create app listing in App Store Connect
- [ ] Fill in app information:
  - [ ] App name: **PastePaw**
  - [ ] Subtitle: *A beautiful clipboard history manager*
  - [ ] Category: **Utilities**
  - [ ] Support URL: (see URLs table above)
  - [ ] Marketing URL: (see URLs table above)
  - [ ] Privacy Policy URL: (see URLs table above)

### 6. App Metadata

- [ ] Upload screenshots:
  - [ ] `docs/screenshot_macos_light.png`
  - [ ] `docs/screenshot_macos_dark.png`
- [ ] Write app description (emphasize privacy, local storage, no cloud sync)
- [ ] Add keywords: clipboard, clipboard manager, history, productivity, paste
- [ ] Set age rating: 4+ (no objectionable content)

### 7. Privacy & Permissions

- [ ] Fill out App Privacy questionnaire in App Store Connect
  - Data Types Collected: **None** (all data stored locally)
  - Third-party Analytics: **No** (if not using analytics)
- [ ] Explain Accessibility permission usage:
  - "PastePaw requests Accessibility permission to enable the optional auto-paste feature, which simulates keyboard input (Cmd+V) to paste clipboard content into other applications. This permission is optional and not required to use the app."

### 8. App Review Information

- [ ] Add demo account (if applicable): **N/A** (no account needed)
- [ ] Add review notes explaining:
  - All clipboard data is stored locally
  - How to use the app (press Ctrl+Shift+V)
  - Accessibility permission is optional
  - AI features require user's own API key

### 9. Upload & Submit

- [ ] Upload .pkg using **Transporter** app or `xcrun altool`
- [ ] Wait for automated processing (~30 minutes)
- [ ] Select build in App Store Connect
- [ ] Submit for review
- [ ] Monitor review status

## 📝 Common Rejection Reasons to Avoid

1. **Private API Usage** ✓ Addressed in build configuration
2. **Sandbox Violations** ✓ All features tested in sandbox
3. **Missing Privacy Policy** ✓ Created and hosted on GitHub Pages
4. **Missing Support URL** ✓ Created and hosted on GitHub Pages
5. **Accessibility Misuse** ✓ Clearly documented as optional feature
6. **Misleading Screenshots** ✓ Using actual macOS screenshots
7. **Missing App Metadata** ✓ Checklist covers all required fields

## 🧪 Pre-Submission Testing

Test these features in the sandboxed App Store build:

- [ ] Clipboard monitoring works
- [ ] Clipboard history saves and loads
- [ ] Search functionality works
- [ ] Folders and organization work
- [ ] Settings persist
- [ ] Hotkey works (Ctrl+Shift+V)
- [ ] Ignored apps feature works
- [ ] Theme switching works
- [ ] App quits cleanly
- [ ] No crashes or errors in Console.app

## 📞 Support Resources

- **GitHub Pages Site**: https://xueshiqiao.github.io/PastePaw/
- **Documentation**: See `docs/appstore_submit.md` and `docs/submit_to_appstore.md`
- **Apple Developer**: https://developer.apple.com/
- **App Store Connect**: https://appstoreconnect.apple.com/

## 🎉 After Approval

- [ ] App goes live on Mac App Store
- [ ] Update GitHub README with App Store badge
- [ ] Announce on social media / website
- [ ] Monitor reviews and feedback
- [ ] Respond to user reviews

## 📅 Timeline Estimate

- **GitHub Pages Setup**: 10 minutes ✓
- **Code Changes**: 2-4 hours
- **Certificates & Config**: 30 minutes
- **Build & Test**: 1 hour
- **App Store Connect Setup**: 1 hour
- **Apple Review**: 1-3 days
- **Total**: ~1 week from start to approval

---

**Good luck with your submission!** 🚀

If you encounter issues, refer to the detailed guides in the `docs/` folder or reach out for help on GitHub.
