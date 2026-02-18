import ReactDOM from 'react-dom/client';
import App from './App';
import { SettingsWindow } from './windows/SettingsWindow';
import { attachConsole } from '@tauri-apps/plugin-log';
import { isMacOS } from './utils/platform';
import './i18n/config'; // Initialize i18n
import './index.css';

attachConsole()
  .then(() => console.log('[WinPaste] Tauri console attached successfully'))
  .catch((err) => console.error('[WinPaste] Failed to attach Tauri console:', err));
console.log('[WinPaste] Frontend loaded - if you see this, DevTools is working!');

if (isMacOS()) {
  document.documentElement.classList.add('macos');
}

const urlParams = new URLSearchParams(window.location.search);
const windowType = urlParams.get('window');

ReactDOM.createRoot(document.getElementById('root')!).render(
  windowType === 'settings' ? <SettingsWindow /> : <App />
);
