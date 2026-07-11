<script setup lang="ts">
import { computed } from 'vue'
import { useChannelStore } from '../../stores/channel'
import type { ChannelPathNodeInput } from '../../types/api'

const props = defineProps<{
  modelValue: ChannelPathNodeInput[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: ChannelPathNodeInput[]]
}>()

const channelStore = useChannelStore()

const channelLevels = computed(() => {
  const levels: Map<number, number[]> = new Map()
  for (const node of props.modelValue) {
    if (!levels.has(node.position)) {
      levels.set(node.position, [])
    }
    levels.get(node.position)!.push(node.channel_id)
  }
  return levels
})

const maxPosition = computed(() => {
  if (props.modelValue.length === 0) return -1
  return Math.max(...props.modelValue.map(n => n.position))
})

function addLevel() {
  const newPosition = maxPosition.value + 1
  const newNode: ChannelPathNodeInput = {
    position: newPosition,
    channel_id: 0,
    status: 'default',
  }
  emit('update:modelValue', [...props.modelValue, newNode])
}

function removeLevel(position: number) {
  const filtered = props.modelValue.filter(n => n.position !== position)
  // Re-index positions
  const reindexed = filtered.map((n, idx) => ({ ...n, position: idx }))
  emit('update:modelValue', reindexed)
}

function setChannel(position: number, channelId: number) {
  const updated = props.modelValue.map(n => {
    if (n.position === position) {
      return { ...n, channel_id: channelId }
    }
    return n
  })
  emit('update:modelValue', updated)
}

function addChannelToLevel(position: number, channelId: number) {
  const newNode: ChannelPathNodeInput = {
    position,
    channel_id: channelId,
    status: 'default',
  }
  emit('update:modelValue', [...props.modelValue, newNode])
}

function removeChannelFromLevel(position: number, channelId: number) {
  const filtered = props.modelValue.filter(n => !(n.position === position && n.channel_id === channelId))
  emit('update:modelValue', filtered)
}

function isLastLevel(position: number): boolean {
  return position === maxPosition.value
}

function getChannelsForLevel(position: number): number[] {
  return channelLevels.value.get(position) || []
}

function channelName(id: number): string {
  return channelStore.channels.find(c => c.id === id)?.name || `渠道 #${id}`
}
</script>

<template>
  <div class="channel-path-input">
    <div v-for="pos in maxPosition + 1" :key="pos - 1" class="level-row">
      <div class="level-label">第 {{ pos }} 级</div>
      <div class="level-channels">
        <span
          v-for="chId in getChannelsForLevel(pos - 1)"
          :key="chId"
          class="channel-chip"
        >
          {{ channelName(chId) }}
          <button v-if="isLastLevel(pos - 1) && getChannelsForLevel(pos - 1).length > 1" @click="removeChannelFromLevel(pos - 1, chId)">×</button>
        </span>
        <select
          v-if="isLastLevel(pos - 1) || getChannelsForLevel(pos - 1).length === 0"
          @change="e => {
            const val = Number((e.target as HTMLSelectElement).value)
            if (val) {
              if (isLastLevel(pos - 1)) {
                addChannelToLevel(pos - 1, val)
              } else {
                setChannel(pos - 1, val)
              }
              ;(e.target as HTMLSelectElement).value = ''
            }
          }"
        >
          <option value="">+ 添加渠道</option>
          <option
            v-for="ch in channelStore.channels"
            :key="ch.id"
            :value="ch.id"
            :disabled="getChannelsForLevel(pos - 1).includes(ch.id)"
          >
            {{ ch.name }}
          </option>
        </select>
      </div>
      <button
        v-if="!isLastLevel(pos - 1) || getChannelsForLevel(pos - 1).length > 0"
        class="remove-level-btn"
        @click="removeLevel(pos - 1)"
      >
        ×
      </button>
    </div>

    <button class="add-level-btn" @click="addLevel">+ 添加链路级别</button>
  </div>
</template>

<style scoped>
.channel-path-input {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.level-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  padding: 0.5rem;
}

.level-label {
  color: var(--text-muted);
  font-size: 0.75rem;
  min-width: 50px;
}

.level-channels {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  flex: 1;
  align-items: center;
}

.channel-chip {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  background: var(--accent, #646cff);
  color: #fff;
  padding: 0.25rem 0.5rem;
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

.level-channels select {
  background: transparent;
  border: 1px dashed var(--border);
  border-radius: 0.25rem;
  padding: 0.25rem 0.5rem;
  color: var(--text-muted);
  font-size: 0.75rem;
  cursor: pointer;
}

.level-channels select:hover {
  border-color: var(--accent, #646cff);
  color: var(--accent, #646cff);
}

.remove-level-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 1.25rem;
  cursor: pointer;
  padding: 0.25rem;
  line-height: 1;
}

.remove-level-btn:hover {
  color: #e74c3c;
}

.add-level-btn {
  background: var(--card-bg-alt, #252525);
  border: 1px dashed var(--border);
  border-radius: 0.5rem;
  padding: 0.5rem;
  color: var(--text-muted);
  font-size: 0.875rem;
  cursor: pointer;
  transition: border-color 0.15s;
}

.add-level-btn:hover {
  border-color: var(--accent, #646cff);
  color: var(--accent, #646cff);
}
</style>
