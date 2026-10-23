<script setup lang="ts">
import type { Difficulty, Mode, TaskDetail } from '~/types/Task'

const emit = defineEmits<{
  generated: [task: TaskDetail] // нейросеть написала задачу — список надо обновить
}>()

const { apiUrl, apiFetch } = useApi()
const { isLoggedIn } = useAuth()

const { data: llmStatus } = await useFetch<{ configured: boolean, model: string | null }>(apiUrl('/llm/status'))

const modes = [
  { label: 'JavaScript', id: 'javascript' },
  { label: 'Python', id: 'python' },
] as const

const difficulties = [
  { label: 'Лёгкая', id: 'easy' },
  { label: 'Средняя', id: 'medium' },
  { label: 'Сложная', id: 'hard' },
] as const

const selectedMode = ref<Mode>('javascript')
const selectedDifficulty = ref<Difficulty>('easy')
const isGenerating = ref(false)
const status = ref<{ type: 'error' | 'success', text: string, taskId?: number } | null>(null)

const handleGenerate = async () => {
  isGenerating.value = true
  status.value = null

  try {
    const task = await apiFetch<TaskDetail>('/tasks/generate', {
      method: 'POST',
      body: { mode: selectedMode.value, difficulty: selectedDifficulty.value },
    })
    status.value = { type: 'success', text: `Готово: «${task.title}»`, taskId: task.id }
    emit('generated', task)
  } catch (err) {
    status.value = { type: 'error', text: errorMessage(err, 'Сервер недоступен') }
  } finally {
    isGenerating.value = false
  }
}
</script>

<template>
  <div class="bg-panel border border-line rounded-card p-6 mb-10">
    <div class="flex flex-col lg:flex-row lg:items-center gap-4">
      <div class="lg:mr-auto">
        <h2 class="mb-1">Новая задача от нейросети</h2>
        <p v-if="!llmStatus?.configured" class="text-sm text-muted">
          Нейросеть не подключена — добавь <code class="font-mono">LLM_API_KEY</code> в <code class="font-mono">backend/.env</code>
        </p>
        <p v-else-if="!isLoggedIn" class="text-sm text-muted">
          <NuxtLink to="/login" class="text-accent underline">Войди</NuxtLink>, чтобы генерировать задачи
        </p>
        <p v-else class="text-sm text-muted">
          {{ llmStatus.model }} напишет задачу и тесты, а потом сама её решит — это и будет соперник
        </p>
      </div>

      <template v-if="llmStatus?.configured && isLoggedIn">
        <select v-model="selectedMode" class="px-4 py-2 rounded-lg bg-raised border border-line text-sm outline-none focus:border-accent">
          <option v-for="mode in modes" :key="mode.id" :value="mode.id">{{ mode.label }}</option>
        </select>
        <select v-model="selectedDifficulty" class="px-4 py-2 rounded-lg bg-raised border border-line text-sm outline-none focus:border-accent">
          <option v-for="difficulty in difficulties" :key="difficulty.id" :value="difficulty.id">{{ difficulty.label }}</option>
        </select>
        <button
          :disabled="isGenerating"
          class="px-5 py-2 bg-accent text-white rounded-lg hover:brightness-110 transition text-sm font-medium disabled:opacity-50"
          @click="handleGenerate"
        >
          {{ isGenerating ? 'Пишет задачу… (до минуты)' : 'Сгенерировать' }}
        </button>
      </template>
    </div>

    <p v-if="status" :class="['text-sm mt-4', status.type === 'error' ? 'text-bad' : 'text-ok']">
      {{ status.text }}
      <NuxtLink v-if="status.taskId" :to="`/task/${status.taskId}`" class="underline ml-1">Открыть →</NuxtLink>
    </p>
  </div>
</template>
