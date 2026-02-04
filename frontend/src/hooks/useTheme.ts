import { useEffect, useState } from 'react';

export function useTheme(theme: string) {
  const [effectiveTheme, setEffectiveTheme] = useState<'light' | 'dark'>(
    theme === 'system'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light'
      : (theme as 'light' | 'dark')
  );

  useEffect(() => {
    const root = window.document.documentElement;
    root.classList.remove('light', 'dark');

    const getSystemTheme = () =>
      window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';

    const applyTheme = (t: string) => {
      const currentSystemTheme = getSystemTheme();
      if (t === 'system') {
        root.classList.add(currentSystemTheme);
        setEffectiveTheme(currentSystemTheme);
      } else {
        root.classList.add(t);
        setEffectiveTheme(t as 'light' | 'dark');
      }
    };

    applyTheme(theme);

    if (theme === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const handleChange = () => {
        root.classList.remove('light', 'dark');
        applyTheme('system');
      };

      mediaQuery.addEventListener('change', handleChange);
      return () => mediaQuery.removeEventListener('change', handleChange);
    }
  }, [theme]);

  return effectiveTheme;
}
