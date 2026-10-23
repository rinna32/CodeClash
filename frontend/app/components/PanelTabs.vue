<script setup lang="ts">
// Вкладки над панелью рабочего места, как в LeetCode: «Описание», «Ответ и разбор».
// Справа в слот можно положить свои кнопки
const props = defineProps<{
  tabs: readonly { id: string, label: string, disabled?: boolean }[]
}>()

const active = defineModel<string>({ required: true })
</script>

<template>
  <div class="flex items-center gap-1 px-2 h-10 border-b border-line bg-raised/40 shrink-0">
    <button
      v-for="tab in props.tabs"
      :key="tab.id"
      :disabled="tab.disabled"
      :class="[
        'px-3 py-1 rounded-md text-sm transition-colors duration-200 disabled:opacity-40 disabled:cursor-not-allowed',
        active === tab.id ? 'bg-raised text-ink font-medium' : 'text-muted hover:text-ink'
      ]"
      @click="active = tab.id"
    >
      {{ tab.label }}
    </button>

    <div class="ml-auto flex items-center gap-2">
      <slot />
    </div>
  </div>
</template>
