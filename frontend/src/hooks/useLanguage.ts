import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

export function useLanguage(language?: string) {
  const { i18n } = useTranslation();

  useEffect(() => {
    if (language && i18n.language !== language) {
      i18n.changeLanguage(language);
      // Store in localStorage for persistence
      localStorage.setItem('pastepaw_language', language);
    }
  }, [language, i18n]);

  const changeLanguage = async (newLang: string) => {
    await i18n.changeLanguage(newLang);
    localStorage.setItem('pastepaw_language', newLang);
  };

  return {
    currentLanguage: i18n.language,
    changeLanguage,
    t: i18n.t,
  };
}
