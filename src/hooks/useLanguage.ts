import { computed } from 'vue'
import { deDE, enUS, frFR, itIT, koKR, ruRU, zhCN, zhTW } from 'naive-ui'
import { useAppStore } from '@/store'
import { setLocale } from '@/locales'

export function getLanguage() {
  const appStore = useAppStore()
  let lang = appStore.language as string
  if (lang === 'auto')
    lang = window?.navigator?.language
  return lang
}

export function useLanguage() {
  const lang = getLanguage()
  const language = computed(() => {
    switch (lang) {
      case 'de-DE':
        setLocale('de-DE')
        return deDE
      case 'it-IT':
        setLocale('it-IT')
        return itIT
      case 'fr-FR':
        setLocale('fr-FR')
        return frFR
      case 'ko-KR':
        setLocale('ko-KR')
        return koKR
      case 'zh-CN':
      case 'zh-SG':
        setLocale('zh-CN')
        return zhCN
      case 'zh-TW':
        setLocale('zh-TW')
        return zhTW
      case 'ru-RU':
        setLocale('ru-RU')
        return ruRU
      default:
        setLocale('en-US')
        return enUS
    }
  })

  return { language }
}
