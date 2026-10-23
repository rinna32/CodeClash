<script setup lang="ts">
const route = useRoute()
const { user, isLoggedIn, logout } = useAuth()
const { theme, toggleTheme } = useTheme()

const links = [
  { label: 'Задачи', to: '/' },
  { label: 'Играть', to: '/play' },
  { label: 'Рейтинг', to: '/leaderboard' },
] as const

const isActive = (to: string) => (to === '/' ? route.path === '/' || route.path.startsWith('/task') : route.path.startsWith(to))
</script>

<template>
  <!-- Тонкая тёмная шапка на светлой странице, как на образце. Цвета свои, а не токены: шапка тёмная при любой теме.
       Тонкая — рабочему месту задачи нужна вся высота экрана -->
  <header class="bg-[#1c1c1c] text-white">
    <div class="px-4 lg:px-6 h-12 flex items-center gap-6">
      <NuxtLink to="/" class="font-display text-lg tracking-wide">
        Code<span class="text-[#e0233f]">Clash</span>
      </NuxtLink>

      <!-- Навигация -->
      <nav class="hidden sm:flex items-center h-full gap-1">
        <NuxtLink
          v-for="link in links"
          :key="link.to"
          :to="link.to"
          :class="[
            'h-full flex items-center px-3 text-sm border-b-2 transition-colors duration-200',
            isActive(link.to) ? 'border-[#e0233f] text-white' : 'border-transparent text-white/60 hover:text-white'
          ]"
        >
          {{ link.label }}
        </NuxtLink>
      </nav>

      <div class="ml-auto flex items-center gap-4">
        <!-- Светлая / тёмная тема -->
        <button
          class="w-8 h-8 flex items-center justify-center rounded-lg text-white/60 hover:text-white hover:bg-white/10 transition-colors duration-200"
          :aria-label="theme === 'dark' ? 'Светлая тема' : 'Тёмная тема'"
          :title="theme === 'dark' ? 'Светлая тема' : 'Тёмная тема'"
          @click="toggleTheme"
        >
          <!-- Солнце — в тёмной теме (переключит на светлую), луна — в светлой -->
          <svg v-if="theme === 'dark'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
          </svg>
          <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z" />
          </svg>
        </button>

        <!-- Аккаунт -->
        <template v-if="isLoggedIn">
          <NuxtLink to="/profile" class="flex items-center gap-2 text-sm hover:text-white/70 transition-colors duration-200">
            <span class="w-7 h-7 rounded-full bg-white/10 flex items-center justify-center text-xs font-semibold uppercase">
              {{ user?.username.slice(0, 1) }}
            </span>
            <span class="hidden sm:inline">{{ user?.username }}</span>
          </NuxtLink>
          <button class="text-sm text-white/60 hover:text-white transition-colors duration-200" @click="logout">
            Выйти
          </button>
        </template>
        <NuxtLink
          v-else
          to="/login"
          class="px-4 py-1.5 bg-accent text-white rounded-lg hover:brightness-110 transition text-sm font-medium"
        >
          Войти
        </NuxtLink>
      </div>
    </div>

    <!-- Навигация на телефоне -->
    <nav class="sm:hidden flex gap-1 px-4 pb-2">
      <NuxtLink
        v-for="link in links"
        :key="link.to"
        :to="link.to"
        :class="[
          'px-3 py-1 rounded-md text-sm transition-colors duration-200',
          isActive(link.to) ? 'bg-white/10 text-white' : 'text-white/60'
        ]"
      >
        {{ link.label }}
      </NuxtLink>
    </nav>
  </header>
</template>
