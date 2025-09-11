<template>
  <div
    v-bind="$attrs"
    :class="['relative', fullWidth ? 'block w-full' : 'inline-block min-w-[22ch]']"
  >
    <!-- Native date input: keeps browser picker UX; locale steered via lang -->
    <input
      type="date"
      class="bordered-field w-full"
      :id="id"
      :lang="langAttr || undefined"
      :value="modelValue ?? ''"
      @input="onInput"
      @change="onChange"
    />
  </div>
</template>

<script setup lang="ts">
defineOptions({ inheritAttrs: false });
import { computed } from 'vue';
import { db } from '../db.ts';

const props = defineProps<{ 
  modelValue: string | null
  id?: string
  fullWidth?: boolean
  modelModifiers?: Record<string, boolean>
  forceLang?: string
}>();

const emit = defineEmits<{ (e: 'update:modelValue', value: string | null): void }>();

const isLazy = computed(() => !!props.modelModifiers?.lazy);
const fullWidth = computed(() => !!props.fullWidth);

// Map configured format to a locale that controls the native date input layout
// - en-US: MM/DD/YYYY
// - en-GB: DD/MM/YYYY
// - en-CA: YYYY-MM-DD (ISO-like)
const langAttr = computed(() => {
  if (props.forceLang) return props.forceLang;
  const style = db.metadata.date_style ?? 'YYYY-MM-DD';
  switch (style) {
    case 'YYYY-MM-DD':
      return 'en-CA';
    case 'D MMM YYYY':
    case 'D MMMM YYYY':
    case 'DD/MM/YYYY':
      return 'en-GB';
    case 'MM/DD/YYYY':
    case 'MMM D, YYYY':
    case 'MMM Do, YYYY':
    case 'MMMM Do, YYYY':
      return 'en-US';
    default:
      // Heuristics for unlisted custom formats
      if (style.includes('DD/MM')) return 'en-GB';
      if (style.includes('MM/DD')) return 'en-US';
      if (style.includes('YYYY') && style.includes('-')) return 'en-CA';
      return undefined; // fall back to document/page language
  }
});

function onInput(e: Event) {
  if (isLazy.value) return; // Defer to change event when lazy
  const target = e.target as HTMLInputElement;
  emit('update:modelValue', target.value || null);
}

function onChange(e: Event) {
  const target = e.target as HTMLInputElement;
  emit('update:modelValue', target.value || null);
}
</script>

<style scoped>
/* The input fills the wrapper width; width is controlled by wrapper classes */
</style>
