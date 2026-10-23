<script setup lang="ts">
import type { TaskDetail } from '~/types/Task'

const props = defineProps<{
  task: TaskDetail
}>()

const statementHtml = computed(() => formatStatement(props.task.statement_md))

// Ввод примера с именами аргументов, как в LeetCode: nums = [2, 7], target = 9
const formatInput = (args: unknown[]) =>
  args
    .map((arg, index) => {
      const name = props.task.params[index]
      const value = JSON.stringify(arg)
      return name ? `${name} = ${value}` : value
    })
    .join(', ')
</script>

<template>
  <div>
    <!-- Условие: v-html безопасен, formatStatement экранирует HTML -->
    <p class="leading-relaxed text-ink/90" v-html="statementHtml" />

    <div v-for="(example, index) in props.task.examples" :key="index" class="mt-6">
      <p class="font-semibold mb-2">Пример {{ index + 1 }}</p>
      <div class="border-l-2 border-line pl-4 font-mono text-sm space-y-1">
        <p><span class="text-muted">Ввод:</span> {{ formatInput(example.args) }}</p>
        <p><span class="text-muted">Вывод:</span> {{ JSON.stringify(example.expected) }}</p>
      </div>
    </div>

    <p v-if="props.task.hidden_tests_count" class="text-sm text-muted mt-6">
      + скрытых тестов при отправке: {{ props.task.hidden_tests_count }}
    </p>
  </div>
</template>
