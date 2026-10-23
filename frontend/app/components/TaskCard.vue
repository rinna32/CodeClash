<script setup lang="ts">
import type { TaskListItem } from '~/types/Task'

const props = defineProps<{
  task: TaskListItem
}>()
</script>

<template>
  <NuxtLink
    :to="`/task/${props.task.id}`"
    class="group block bg-panel rounded-card p-6 border border-line hover:border-accent hover:shadow-[0_20px_60px_rgba(0,0,0,0.08)] transition-all duration-300"
  >
    <div class="flex items-center gap-2 mb-4">
      <span class="px-3 py-1 rounded-full bg-ink text-bg text-xs font-medium">
        {{ MODE_LABELS[props.task.mode] }}
      </span>
      <span :class="['px-3 py-1 rounded-full text-xs font-medium', DIFFICULTY_COLORS[props.task.difficulty]]">
        {{ DIFFICULTY_LABELS[props.task.difficulty] }}
      </span>
      <span
        v-if="props.task.source === 'llm'"
        class="ml-auto px-1.5 py-0.5 rounded bg-accent/15 text-accent text-[10px] font-semibold"
        title="Задачу написала нейросеть"
      >
        ИИ
      </span>
    </div>

    <h3 class="text-lg mb-6">{{ props.task.title }}</h3>

    <span class="text-sm text-muted group-hover:text-accent transition-colors duration-200">
      Решать →
    </span>
  </NuxtLink>
</template>
