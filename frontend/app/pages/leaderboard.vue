<script setup lang="ts">
import type { LeaderboardItem } from '~/types/Auth'
import type { Mode } from '~/types/Task'

const { apiUrl } = useApi()
const { user } = useAuth()

const modeTabs = [
  { label: 'JavaScript', id: 'javascript' },
  { label: 'Python', id: 'python' },
] as const

const selectedMode = ref<Mode>('javascript')
const query = computed(() => ({ mode: selectedMode.value }))

const { data: leaders, pending, error } = await useFetch<LeaderboardItem[]>(apiUrl('/leaderboard'), {
  query,
  default: () => [],
})

// Призовые места — золото, серебро, бронза
const PLACE_COLORS: Record<number, string> = {
  1: 'text-[#b8860b] dark:text-[#ffd700]', // на светлом яркое золото не видно — берём тёмное
  2: 'text-[#808080] dark:text-[#c0c0c0]',
  3: 'text-[#a0522d] dark:text-[#cd7f32]',
}

useHead({ title: 'Рейтинг — CodeClash' })
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 lg:px-6 py-8">
    <h1 class="text-2xl font-semibold mb-1">Рейтинг</h1>
    <p class="text-muted mb-6">Лучшие игроки по каждому режиму.</p>

    <div class="inline-flex bg-panel border border-line rounded-lg p-1 gap-1 mb-4">
      <button
        v-for="tab in modeTabs"
        :key="tab.id"
        :class="[
          'px-3 py-1 rounded-md text-sm transition-colors duration-200',
          selectedMode === tab.id ? 'bg-raised text-ink font-medium' : 'text-muted hover:text-ink'
        ]"
        @click="selectedMode = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>

    <div v-if="pending" class="bg-panel border border-line rounded-card text-center py-16">
      <p class="text-muted">Загружаем рейтинг...</p>
    </div>

    <div v-else-if="error" class="bg-panel border border-line rounded-card text-center py-16">
      <p class="text-muted">Не получилось загрузить рейтинг — запущен ли бэкенд?</p>
    </div>

    <div v-else-if="!leaders.length" class="bg-panel border border-line rounded-card text-center py-16">
      <p class="text-muted mb-4">В этом режиме ещё никто не играл</p>
      <NuxtLink to="/play" class="px-6 py-2.5 bg-accent text-white rounded-lg hover:brightness-110 transition text-sm font-medium">
        Стать первым →
      </NuxtLink>
    </div>

    <div v-else class="bg-panel border border-line rounded-card overflow-hidden">
      <div class="flex items-center gap-4 px-4 h-10 text-xs text-muted border-b border-line">
        <span class="w-8">#</span>
        <span>Игрок</span>
        <span class="ml-auto hidden sm:inline">Победы</span>
        <span class="w-16 text-right">Рейтинг</span>
      </div>
      <div
        v-for="leader in leaders"
        :key="leader.username"
        :class="[
          'flex items-center gap-4 px-4 h-12 text-sm',
          leader.username === user?.username ? 'bg-accent/10' : 'even:bg-raised/30'
        ]"
      >
        <span :class="['w-8 font-mono font-semibold', PLACE_COLORS[leader.place] ?? 'text-muted']">{{ leader.place }}</span>
        <span class="font-medium">{{ leader.username }}</span>
        <span :class="['px-2.5 py-0.5 rounded-full text-xs font-semibold', LEAGUE_COLORS[leader.league]]">{{ leader.league }}</span>
        <span class="ml-auto text-muted hidden sm:inline">{{ leader.wins }} из {{ leader.games }}</span>
        <span class="w-16 text-right font-mono">{{ leader.rating }}</span>
      </div>
    </div>
  </div>
</template>
