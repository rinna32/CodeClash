// Светлая / тёмная тема. Храним в cookie, а не в localStorage: её видит и сервер,
// поэтому страница сразу приходит в нужной теме, без вспышки светлой
export type Theme = 'light' | 'dark'

const THEME_MAX_AGE = 365 * 24 * 60 * 60 // с — запоминаем на год

export const useTheme = () => {
  const theme = useCookie<Theme>('cc_theme', { default: () => 'light', maxAge: THEME_MAX_AGE, sameSite: 'lax' })

  const toggleTheme = () => {
    theme.value = theme.value === 'dark' ? 'light' : 'dark'
  }

  return {
    theme,
    toggleTheme,
  }
}
