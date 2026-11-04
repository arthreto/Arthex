import { moduleTranslator } from '../../core/i18n';
import type en from './locales/en.json';

export const t = moduleTranslator<typeof en>('file-search');
