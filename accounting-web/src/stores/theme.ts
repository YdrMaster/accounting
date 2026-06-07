import { defineStore } from 'pinia'
import { ref } from 'vue'

type Theme = 'light' | 'dark' | 'auto'

function getSystemTheme() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function getSavedTheme(): Theme {
  const raw = localStorage.getItem('theme')
  if (raw === 'light' || raw === 'dark' || raw === 'auto') {
    return raw
  }
  // 非法值自动清理
  if (raw !== null) {
    localStorage.removeItem('theme')
  }
  return 'auto'
}

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<Theme>(getSavedTheme())
  const isDark = ref(theme.value === 'dark' || (theme.value === 'auto' && getSystemTheme() === 'dark'))

  function apply() {
    const raw = theme.value
    // 防御：如果 theme.value 意外变成对象，回退到 'auto'
    const themeName = typeof raw === 'string' ? (raw as Theme) : 'auto'
    const dark = themeName === 'dark' || (themeName === 'auto' && getSystemTheme() === 'dark')
    isDark.value = dark
    document.documentElement.classList.toggle('dark', dark)
  }

  function setTheme(value: Theme) {
    theme.value = value
    localStorage.setItem('theme', value)
    apply()
  }

  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  mediaQuery.addEventListener('change', () => {
    if (theme.value === 'auto') {
      apply()
    }
  })

  apply()

  return { theme, isDark, setTheme }
})
