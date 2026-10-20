import { memo } from 'react';
import { useTranslation } from 'react-i18next';

export const Chats = memo(function Chats() {
  const { t } = useTranslation();

  return (
    <section className="flex h-full flex-col px-6 py-5">
      <h1 className="text-[22px] font-semibold tracking-tight text-white/90">
        {t('chat.nav.title')}
      </h1>
    </section>
  );
});
