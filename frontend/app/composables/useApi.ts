// Запросы к API. Базовый адрес — из runtimeConfig, токен входа — из cookie,
// чтобы не повторять их в каждом файле.
export const useApi = () => {
  const config = useRuntimeConfig()
  const token = useCookie<string | null>('cc_token')
  const user = useCookie<object | null>('cc_user')

  const apiUrl = (path: string) => `${config.public.apiBase}${path}`

  // Для useFetch: заголовок пересчитается сам, если токен сменится
  const authHeaders = computed(() => {
    const headers: Record<string, string> = {}
    if (token.value) headers.Authorization = `Bearer ${token.value}`
    return headers
  })

  // Разовые запросы из обработчиков (кнопки «Отправить», «Войти» и т.п.)
  const apiFetch = <T>(path: string, options: { method?: 'GET' | 'POST', body?: object } = {}) =>
    $fetch<T>(apiUrl(path), {
      ...options,
      headers: authHeaders.value,
      // 401 — токен протух или аккаунта больше нет: забываем вход, иначе в шапке висит старое имя
      onResponseError: ({ response }) => {
        if (response.status !== 401) return
        token.value = null
        user.value = null
      },
    })

  return {
    apiUrl,
    apiFetch,
    authHeaders,
  }
}
