import { useSettingsStore } from '~/stores/settings'

export function useTheme() {
  const settingsStore = useSettingsStore()

  const applyTheme = async (theme: string) => {
    const html = document.documentElement
    
    // 移除现有主题类
    html.classList.remove('light', 'dark')
    
    if (theme === 'auto') {
      // 跟随系统
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const prefersDark = mediaQuery.matches
      html.classList.add(prefersDark ? 'dark' : 'light')
      
      // 监听系统主题变化
      const handleChange = (e: MediaQueryListEvent) => {
        html.classList.remove('light', 'dark')
        html.classList.add(e.matches ? 'dark' : 'light')
      }
      mediaQuery.addEventListener('change', handleChange)
    } else {
      html.classList.add(theme)
    }
  }

  const initTheme = async () => {
    const settings = await settingsStore.getSettings()
    const theme = settings.theme || 'light'
    await applyTheme(theme)
  }

  const setTheme = async (theme: string) => {
    await settingsStore.saveSettings({ theme })
    await applyTheme(theme)
  }

  return {
    applyTheme,
    initTheme,
    setTheme
  }
}

