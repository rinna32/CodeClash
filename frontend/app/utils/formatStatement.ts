// Условие задачи приходит в markdown, но из разметки там только `код`.
// Полноценный markdown-парсер ради этого не тянем: экранируем HTML и подсвечиваем код.
export function formatStatement(text: string) {
  const escaped = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')

  return escaped.replace(
    /`([^`]+)`/g,
    '<code class="px-1.5 py-0.5 rounded bg-raised text-ink font-mono text-[0.9em]">$1</code>',
  )
}
