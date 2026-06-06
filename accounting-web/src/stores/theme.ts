import { defineStore } from 'pinia'
import { ref } from 'vue'

type Theme = 'light' | 'dark' | 'auto'

function getSystemTheme() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<Theme>((localStorage.getItem('theme') as Theme) || 'auto')
  const isDark = ref(theme.value === 'dark' || (theme.value === 'auto' && getSystemTheme() === 'dark'))

  function apply() {
    const dark = theme.value === 'dark' || (theme.value === 'auto' && getSystemTheme() === 'dark')
    isDark.value = dark
    if (dark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
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
