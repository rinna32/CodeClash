// Страницы только для вошедших: без аккаунта отправляем на вход и возвращаем обратно после
export default defineNuxtRouteMiddleware((to) => {
  const { isLoggedIn } = useAuth()

  if (!isLoggedIn.value) {
    return navigateTo({ path: '/login', query: { redirect: to.fullPath } })
  }
})
