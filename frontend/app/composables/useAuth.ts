// Аккаунт: токен и пользователь лежат в cookie — их видит и сервер при SSR, и браузер.
import type { AuthResponse, User } from '~/types/Auth'

const SESSION_MAX_AGE = 7 * 24 * 60 * 60 // с, столько же живёт токен на бэкенде

export const useAuth = () => {
  const token = useCookie<string | null>('cc_token', { maxAge: SESSION_MAX_AGE, sameSite: 'lax' })
  const user = useCookie<User | null>('cc_user', { maxAge: SESSION_MAX_AGE, sameSite: 'lax' })
  const { apiFetch } = useApi()

  const isLoggedIn = computed(() => !!token.value && !!user.value)

  const authenticate = async (path: string, username: string, password: string) => {
    const response = await apiFetch<AuthResponse>(path, {
      method: 'POST',
      body: { username, password },
    })
    token.value = response.token
    user.value = response.user
  }

  const login = (username: string, password: string) => authenticate('/auth/login', username, password)
  const register = (username: string, password: string) => authenticate('/auth/register', username, password)

  const logout = () => {
    token.value = null
    user.value = null
    navigateTo('/')
  }

  return {
    user,
    isLoggedIn,
    login,
    register,
    logout,
  }
}
