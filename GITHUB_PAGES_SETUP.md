# GitHub Pages Quick Setup Guide

## 🚀 Enable GitHub Pages (5 minutes)

### Step 1: Push the docs folder to GitHub

Make sure all the files in the `docs/product_pages/` folder are committed and pushed to your GitHub repository:

```bash
git add docs/product_pages/
git commit -m "Add GitHub Pages site for App Store submission"
git push origin main
```

### Step 2: Enable GitHub Pages

1. Go to your repository on GitHub: https://github.com/XueshiQiao/PastePaw
2. Click **Settings** (top navigation)
3. Click **Pages** (left sidebar)
4. Under **Source**:
   - Select: **Deploy from a branch**
   - Branch: **main** (or **master** if that's your default branch)
   - Folder: **/docs**
5. Click **Save**

### Step 3: Wait for deployment

- GitHub will build and deploy your site (takes 2-3 minutes)
- You'll see a green checkmark when it's ready
- Your site will be live at: **https://xueshiqiao.github.io/PastePaw/**

### Step 4: Verify the pages

Visit these URLs to make sure everything is working:

- ✅ Landing page: https://xueshiqiao.github.io/PastePaw/
- ✅ Privacy policy: https://xueshiqiao.github.io/PastePaw/privacy.html
- ✅ Support page: https://xueshiqiao.github.io/PastePaw/support.html
- ✅ Terms of service: https://xueshiqiao.github.io/PastePaw/terms.html

---

## 📝 URLs for App Store Connect

Once GitHub Pages is enabled, use these URLs when filling out your App Store Connect listing:

### Required URLs

| Field in App Store Connect | URL to Use |
|---------------------------|------------|
| **Support URL** | `https://xueshiqiao.github.io/PastePaw/support.html` |
| **Marketing URL** (optional) | `https://xueshiqiao.github.io/PastePaw/` |
| **Privacy Policy URL** | `https://xueshiqiao.github.io/PastePaw/privacy.html` |

### Where to enter these URLs

1. Log in to [App Store Connect](https://appstoreconnect.apple.com/)
2. Select your app (PastePaw)
3. Go to **App Information** section
4. Scroll to **General Information**
5. Enter the URLs in the corresponding fields

---

## 🎨 Customizing the Site (Optional)

All pages are pure HTML/CSS files. To customize:

1. Edit the `.html` files in the `docs/product_pages/` folder
2. Commit and push changes
3. GitHub Pages will automatically rebuild (takes 1-2 minutes)

### Files you might want to customize:

- **index.html** - Update features, screenshots, or download links
- **privacy.html** - Update contact email or privacy details
- **support.html** - Update FAQ, contact info, or system requirements
- **terms.html** - Update legal information

---

## 📱 Testing on Mobile

All pages are responsive and work on mobile devices. Test by:

1. Opening the URLs on your iPhone/iPad
2. Or use Chrome DevTools: Right-click → Inspect → Toggle device toolbar
3. Verify the layout looks good on different screen sizes

---

## 🔍 SEO & Metadata

The pages include proper meta tags for SEO:
- Page titles
- Meta descriptions
- Open Graph tags (for social media sharing)
- Favicon (app icon)
- Responsive viewport settings

---

## ⚠️ Troubleshooting

### Site not loading after enabling GitHub Pages?

1. Wait 5 minutes (it can take a bit longer sometimes)
2. Check **Settings → Pages** for any error messages
3. Make sure the `docs/product_pages/` folder is in the **main** branch
4. Try a hard refresh: **Cmd + Shift + R** (macOS)

### 404 error on pages?

- Make sure file names are exactly: `privacy.html`, `support.html`, `terms.html`
- File names are case-sensitive
- Check that files were pushed to GitHub

### Images not loading?

- Screenshots should be in the `docs/product_pages/` folder
- File names: `screenshot_macos_light.png` and `screenshot_macos_dark.png`
- Make sure they're committed and pushed

### Custom domain (optional)?

If you want to use a custom domain like `pastepaw.com`:

1. Buy a domain from a registrar
2. Add a `CNAME` file to `docs/product_pages/` with your domain
3. Configure DNS settings at your registrar
4. Update **Settings → Pages → Custom domain**

---

## ✅ Checklist

- [ ] Pushed `docs/product_pages/` folder to GitHub
- [ ] Enabled GitHub Pages in Settings
- [ ] Verified landing page loads
- [ ] Verified privacy policy loads
- [ ] Verified support page loads
- [ ] Verified terms page loads
- [ ] Screenshots display correctly
- [ ] All links work
- [ ] Pages work on mobile
- [ ] Ready to submit URLs to App Store Connect

---

## 🎉 Done!

Your GitHub Pages site is now live and ready for App Store submission!

**Next Steps:**
1. Copy the URLs above
2. Go to App Store Connect
3. Fill in the Support, Marketing, and Privacy Policy URLs
4. Continue with your app submission

For the complete App Store submission process, see [APPSTORE_CHECKLIST.md](APPSTORE_CHECKLIST.md).
