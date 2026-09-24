import { Languages } from 'lucide-react';
import { useI18n } from '../i18n';
import type { LanguageKind } from '../lib/settings';
import { useSettingsStore } from '../stores/useSettingsStore';

export function LanguageSwitcher() {
  const { t } = useI18n();
  const language = useSettingsStore((state) => state.global.language);
  const updateGlobal = useSettingsStore((state) => state.updateGlobal);

  return (
    <label className="language-switcher" title={t('global.language')}>
      <Languages size={15} />
      <select
        aria-label={t('global.language')}
        value={language}
        onChange={(event) => updateGlobal('language', event.target.value as LanguageKind)}
      >
        <option value="de">DE</option>
        <option value="en">EN</option>
        <option value="fr">FR</option>
        <option value="es">ES</option>
      </select>
    </label>
  );
}
