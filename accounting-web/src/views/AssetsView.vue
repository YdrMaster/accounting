<script setup lang="ts">
const debts = [
  { name: '今日免息', amount: -712.84, bank: '江苏银行 49天' },
  { name: '花呗', amount: -712.84, bank: '22天内还款 · 可用：¥92,587.16' },
  { name: '江苏银行', amount: -18289.28, bank: '17天内还款 · 可用：¥41,710.72' },
  { name: '招商银行信用卡', amount: -8706.59, bank: '13天内还款 · 可用：¥11,000.00' },
]
</script>

<template>
  <div class="assets">
    <div class="net-worth">
      <p class="label">净资产</p>
      <p class="amount">¥491,898.10</p>
      <div class="row">
        <span>总资产 ¥536,468.54</span>
        <span>总负债 -¥44,570.44</span>
        <span>分期 -¥721.28</span>
      </div>
    </div>

    <div class="card">
      <h3>资产趋势</h3>
      <div class="progress-track">
        <div class="progress-fill" style="width: 12%"></div>
      </div>
      <div class="row muted">
        <span>9 项资产 | 5 项负债</span>
        <span>负债率 8.31%</span>
      </div>
    </div>

    <div class="card split">
      <div>
        <span class="label">总借入</span>
        <strong>¥0.00</strong>
      </div>
      <div>
        <span class="label">总借出</span>
        <strong>¥0.00</strong>
      </div>
    </div>

    <div class="card">
      <div class="debt-header">
        <span>负债</span>
        <span>-¥44,570.44</span>
      </div>
      <div v-for="debt in debts" :key="debt.name" class="debt-item">
        <div class="info">
          <span class="name">{{ debt.name }}</span>
          <span class="bank">{{ debt.bank }}</span>
        </div>
        <span class="amount">{{ debt.amount.toFixed(2) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.assets {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.net-worth {
  text-align: center;
  background: var(--card-bg-alt);
  border-radius: 1rem;
  padding: 1.5rem;
}

.net-worth .label {
  margin: 0;
  color: var(--text-muted);
}

.net-worth .amount {
  margin: 0.5rem 0;
  font-size: 2rem;
  font-weight: 600;
  color: var(--text-heading);
}

.net-worth .row {
  display: flex;
  justify-content: space-around;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.card {
  background: var(--card-bg-alt);
  border-radius: 1rem;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.card h3 {
  margin: 0;
  color: var(--text-heading);
}

.progress-track {
  height: 0.5rem;
  background: var(--border);
  border-radius: 0.25rem;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #4ade80;
  border-radius: 0.25rem;
}

.row.muted {
  display: flex;
  justify-content: space-between;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.split {
  flex-direction: row;
  justify-content: space-around;
  text-align: center;
}

.split > div {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.split strong {
  color: var(--text-heading);
  font-size: 1.25rem;
}

.debt-header {
  display: flex;
  justify-content: space-between;
  color: var(--text-heading);
  font-weight: 500;
}

.debt-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0;
  border-bottom: 1px solid var(--border);
}

.debt-item .info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.name {
  color: var(--text-heading);
}

.bank {
  color: var(--text-muted);
  font-size: 0.75rem;
}

.amount {
  color: #ff7b7b;
  font-weight: 500;
}
</style>
