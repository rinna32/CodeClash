<script setup lang="ts">
const route = useRoute()
const { login, register } = useAuth()

const mode = ref<'login' | 'register'>('login')
const username = ref('')
const password = ref('')
const status = ref<{ type: 'error', text: string } | null>(null)
const isSending = ref(false)

const tabs = [
  { label: 'Вход', id: 'login' },
  { label: 'Регистрация', id: 'register' },
] as const

const handleSubmit = async () => {
  if (!username.value.trim() || !password.value) {
    status.value = { type: 'error', text: 'Введи имя и пароль' }
    return
  }

  isSending.value = true
  status.value = null

  try {
    if (mode.value === 'login') {
      await login(username.value, password.value)
    } else {
      await register(username.value, password.value)
    }
    // После входа — туда, откуда пришёл (например, со страницы матча)
    const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : '/play'
    await navigateTo(redirect)
  } catch (err) {
    status.value = { type: 'error', text: errorMessage(err, 'Сервер недоступен — запущен ли бэкенд?') }
  } finally {
    isSending.value = false
  }
}
</script>

<template>
  <div class="max-w-md mx-auto px-4 py-16">
    <div class="text-center mb-8">
      <p class="font-display text-3xl tracking-wide mb-2">Code<span class="text-accent">Clash</span></p>
      <p class="text-muted">Аккаунт нужен, чтобы играть с ботом и копить рейтинг.</p>
    </div>

    <div class="bg-panel border border-line rounded-card p-6">
      <!-- Вход / регистрация -->
      <div class="flex bg-bg rounded-lg p-1 gap-1 mb-6">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="[
            'flex-1 py-1.5 rounded-md text-sm font-medium transition-colors duration-200',
            mode === tab.id ? 'bg-raised text-ink' : 'text-muted hover:text-ink'
          ]"
          @click="mode = tab.id; status = null"
        >
          {{ tab.label }}
        </button>
      </div>

      <form class="space-y-4" @submit.prevent="handleSubmit">
        <label class="block">
          <span class="text-sm text-muted">Имя</span>
          <input
            v-model="username"
            autocomplete="username"
            class="mt-1 w-full px-3 py-2.5 rounded-lg bg-raised border border-line focus:border-accent outline-none transition-colors duration-200 placeholder:text-muted/60"
            placeholder="rinna"
          >
        </label>

        <label class="block">
          <span class="text-sm text-muted">Пароль</span>
          <input
            v-model="password"
            type="password"
            :autocomplete="mode === 'login' ? 'current-password' : 'new-password'"
            class="mt-1 w-full px-3 py-2.5 rounded-lg bg-raised border border-line focus:border-accent outline-none transition-colors duration-200 placeholder:text-muted/60"
            placeholder="минимум 6 символов"
          >
        </label>

        <p v-if="status" class="text-sm text-bad">{{ status.text }}</p>

        <button
          type="submit"
          :disabled="isSending"
          class="w-full py-2.5 bg-accent text-white rounded-lg hover:brightness-110 transition text-sm font-semibold disabled:opacity-50"
        >
          {{ isSending ? 'Секунду...' : mode === 'login' ? 'Войти' : 'Создать аккаунт' }}
        </button>
      </form>
    </div>
  </div>
</template>
