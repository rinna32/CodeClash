import type { Config } from 'tailwindcss'

// Дизайн-токены: сами цвета — CSS-переменные в main.css, здесь только привязка к классам.
// Так тему (светлая с бордовым акцентом или тёмная) меняем в одном месте, а в разметке пишем bg-panel, text-muted.
// rgb(... / <alpha-value>) — чтобы работала прозрачность: bg-accent/10, border-bad/40
const token = (name: string) => `rgb(var(--${name}) / <alpha-value>)`

export default <Partial<Config>>{
  // dark: — для редких цветов вне токенов (лиги, медали). Тему ставит атрибут data-theme на <html>
  darkMode: ['class', '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        bg: token('bg'), // фон страницы
        panel: token('panel'), // панели и карточки
        raised: token('raised'), // поля ввода, вкладки, ховер строк
        line: token('line'), // границы
        ink: token('ink'), // основной текст
        muted: token('muted'), // второстепенный текст
        accent: token('accent'), // фирменный бордовый
        ok: token('ok'), // принято, «Отправить», победа
        bad: token('bad'), // ошибка, поражение
        easy: token('easy'),
        medium: token('medium'),
        hard: token('hard'),
      },
      // sans — весь текст, display — заголовки и логотип
      fontFamily: {
        sans: ['Jura', 'system-ui', 'sans-serif'],
        display: ['"Russo One"', 'Jura', 'sans-serif'],
      },
      borderRadius: {
        card: '6px', // почти прямые углы, как у карточек на образце
      },
    },
  },
}
