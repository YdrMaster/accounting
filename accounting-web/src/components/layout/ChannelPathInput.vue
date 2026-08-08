<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useChannelStore } from '../../stores/channel'
import type { ChannelPathNodeInput } from '../../types/api'

const props = defineProps<{
  modelValue: ChannelPathNodeInput[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: ChannelPathNodeInput[]]
}>()

const { t } = useI18n()

const channelStore = useChannelStore()

/** 线性链：忽略 channel_id 为 0 的占位节点，按 position 排序 */
const chain = computed(() =>
  props.modelValue.filter(n => n.channel_id > 0).sort((a, b) => a.position - b.position)
)

/** 候选渠道：已选的不允许重复选择 */
function isChosen(channelId: number): boolean {
  return chain.value.some(n => n.channel_id === channelId)
}

function channelName(id: number): string {
  return (
    channelStore.channels.find(c => c.id === id)?.name || t('channelInput.channelNumber', { id })
  )
}

/** 选中即追加到链尾：每选一个才能选下一个，中间不空不重 */
function appendChannel(e: Event) {
  const select = e.target as HTMLSelectElement
  const channelId = Number(select.value)
  if (!channelId) return
  emit('update:modelValue', [
    ...props.modelValue,
    { position: chain.value.length, channel_id: channelId, status: 'default' },
  ])
  select.value = ''
}

/** 仅链尾可删 */
function popLast() {
  const last = chain.value[chain.value.length - 1]
  if (!last) return
  emit(
    'update:modelValue',
    props.modelValue.filter(n => n !== last)
  )
}
</script>

<template>
  <div class="channel-path-input">
    <div class="path-chain">
      <template v-for="(node, index) in chain" :key="node.position">
        <span v-if="index > 0" class="chain-sep">▸</span>
        <span class="channel-chip">
          {{ channelName(node.channel_id) }}
          <button v-if="index === chain.length - 1" @click="popLast">×</button>
        </span>
      </template>
      <select class="chain-select" @change="appendChannel">
        <option value="">{{ t('channelInput.selectChannel') }}</option>
        <option
          v-for="ch in channelStore.channels"
          :key="ch.id"
          :value="ch.id"
          :disabled="isChosen(ch.id)"
        >
          {{ ch.name }}
        </option>
      </select>
    </div>
  </div>
</template>

<style scoped>
.channel-path-input {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

/* 链路横排：层级之间用 ▸ 分隔 */
.path-chain {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.375rem;
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  padding: 0.5rem;
}

.chain-sep {
  color: var(--text-muted);
  font-size: 0.75rem;
  flex-shrink: 0;
}

.channel-chip {
  display: flex;
  align-items: center;
  gap: 0.5ch;
  background: var(--accent, #646cff);
  color: #fff;
  padding: 0.25rem 0.5ch 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
}

.channel-chip button {
  background: none;
  border: none;
  color: #fff;
  cursor: pointer;
  font-size: 0.875rem;
  line-height: 1;
  opacity: 0.7;
}

.channel-chip button:hover {
  opacity: 1;
}

/* 幽灵芯片风格的内联下拉 */
.chain-select {
  appearance: none;
  background: transparent;
  border: 1px dashed var(--border);
  border-radius: 0.25rem;
  padding: 0.25rem 0.5rem;
  color: var(--text-muted);
  font-size: 0.75rem;
  cursor: pointer;
}

.chain-select:hover {
  border-color: var(--accent, #646cff);
  color: var(--accent, #646cff);
}
</style>
