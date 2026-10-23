// Текст ошибки из ответа бэкенда ({ "error": "..." }) или запасной вариант
export function errorMessage(err: unknown, fallback: string) {
  if (err && typeof err === 'object' && 'data' in err) {
    const data = (err as { data?: { error?: unknown } }).data
    if (typeof data?.error === 'string') return data.error
  }

  return fallback
}

// 401 — токен протух: просим войти заново
export function isUnauthorized(err: unknown) {
  return !!err && typeof err === 'object' && 'statusCode' in err && (err as { statusCode?: number }).statusCode === 401
}
