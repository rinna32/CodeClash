<script setup lang="ts">
import type { Difficulty, Mode, TaskListItem } from '~/types/Task'

const { apiUrl } = useApi()

const modeTabs = [
  { label: 'Все', id: null },
  { label: 'JavaScript', id: 'javascript' },
  { label: 'Python', id: 'python' },
  { label: 'CSS', id: 'css' },
] as const

const difficultyTabs = [
  { label: 'Любая', id: null },
  { label: 'Лёгкая', id: 'easy' },
  { label: 'Средняя', id: 'medium' },
  { label: 'Сложная', id: 'hard' },
] as const

const selectedMode = ref<Mode | null>(null)
const selectedDifficulty = ref<Difficulty | null>(null)

// Пустые фильтры не отправляем: бэкенд ждёт либо значение, либо отсутствие параметра
const query = computed(() => ({
  ...(selectedMode.value && { mode: selectedMode.value }),
  ...(selectedDifficulty.value && { difficulty: selectedDifficulty.value }),
}))

// query реактивный — useFetch сам перезапросит список при смене фильтра
const { data: tasks, pending, error, refresh } = await useFetch<TaskListItem[]>(apiUrl('/tasks'), {
  query,
  default: () => [],
})

const hasActiveFilters = computed(() => selectedMode.value !== null || selectedDifficulty.value !== null)

const clearFilters = () => {
  selectedMode.value = null
  selectedDifficulty.value = null
}
</script>

<template>
  <div class="max-w-7xl mx-auto px-6 py-12">
    <!-- Заголовок -->
    <div class="mb-10">
      <h1 class="text-[clamp(36px,6vw,64px)] leading-[1.05] tracking-[-0.02em] mb-3">
        Задачи
      </h1>
      <p class="text-muted text-lg">
        Тренируйся на задачах или <NuxtLink to="/play" class="text-accent underline">сразись с ботом</NuxtLink>.
      </p>
    </div>

    <GenerateTaskPanel @generated="refresh()" />

    <!-- Фильтры: режим и сложность -->
    <div class="flex flex-col lg:flex-row gap-4 mb-8">
      <div class="flex flex-wrap gap-2">
        <button
          v-for="tab in modeTabs"
          :key="tab.label"
          :class="[
            'px-4 py-2 rounded-full text-sm font-medium transition-colors duration-200',
            selectedMode === tab.id ? 'bg-accent text-white' : 'bg-panel border border-line hover:bg-raised'
          ]"
          @click="selectedMode = tab.id"
        >
          {{ tab.label }}
        </button>
      </div>

      <div class="flex flex-wrap gap-2 lg:ml-auto">
        <button
          v-for="tab in difficultyTabs"
          :key="tab.label"
          :class="[
            'px-4 py-2 rounded-full text-sm font-medium transition-colors duration-200',
            selectedDifficulty === tab.id ? 'bg-accent text-white' : 'bg-panel border border-line hover:bg-raised'
          ]"
          @click="selectedDifficulty = tab.id"
        >
          {{ tab.label }}
        </button>
      </div>
    </div>

    <!-- Состояния: загрузка → ошибка → пусто → список -->
    <div v-if="pending" class="text-center py-20">
      <p class="text-muted">Загружаем задачи...</p>
    </div>

    <div v-else-if="error" class="text-center py-20">
      <p class="text-lg mb-2">Не получилось загрузить задачи</p>
      <p class="text-muted text-sm">Проверь, что бэкенд запущен: <code class="font-mono">cd backend && cargo run</code></p>
    </div>

    <div v-else-if="!tasks.length" class="text-center py-20">
      <p class="text-muted text-lg mb-4">Под такие фильтры задач нет</p>
      <button
        v-if="hasActiveFilters"
        class="px-6 py-3 border border-accent text-accent rounded-lg text-sm font-medium hover:bg-accent hover:text-white transition-colors duration-200"
        @click="clearFilters"
      >
        Сбросить фильтры
      </button>
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      <TaskCard v-for="task in tasks" :key="task.id" :task="task" />
    </div>
  </div>
</template>
