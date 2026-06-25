<script setup lang="ts">
const categories = [
  { name: '正餐', total: 2500, spent: 2493.24 },
  { name: '日用品', total: 600, spent: 2299.83 },
  { name: '日常', total: 1000, spent: 759.12 },
  { name: '会员服务', total: 400, spent: 117.02 },
  { name: '服饰', total: 800, spent: 827.32 },
  { name: '交通', total: 600, spent: 1008.1 },
]
</script>

<template>
  <div class="budget">
    <div class="total-card">
      <div class="info">
        <h3>总预算</h3>
        <p>总额：6,000.00 | 支出：16,320.59</p>
        <p>有 18,291.24 不计入预算</p>
      </div>
      <div class="ring over">
        <span>超支</span>
        <strong>10,320.59</strong>
      </div>
    </div>

    <h3 class="section-title">分类预算</h3>
    <div class="category-list">
      <div v-for="cat in categories" :key="cat.name" class="category">
        <div class="meta">
          <span class="name">{{ cat.name }}</span>
          <span class="amount">总额：{{ cat.total }} | {{ cat.spent.toFixed(2) }}</span>
        </div>
        <div class="ring" :class="{ over: cat.spent > cat.total }">
          <span>{{ cat.spent > cat.total ? '超支' : '剩余' }}</span>
          <strong>{{ Math.abs(cat.total - cat.spent).toFixed(2) }}</strong>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.budget {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.total-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--card-bg-alt);
  border-radius: 1rem;
  padding: 1rem;
}

.total-card h3 {
  margin: 0 0 0.5rem;
  color: var(--text-heading);
}

.total-card p {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.section-title {
  margin: 0;
  color: var(--text-heading);
}

.category-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.category {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--card-bg-alt);
  border-radius: 0.75rem;
  padding: 0.75rem 1rem;
}

.meta {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.name {
  color: var(--text-heading);
  font-weight: 500;
}

.amount {
  color: var(--text-muted);
  font-size: 0.875rem;
}

.ring {
  width: 4rem;
  height: 4rem;
  border-radius: 50%;
  border: 0.25rem solid #4ade80;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 0.625rem;
  color: var(--text-muted);
}

.ring.over {
  border-color: #ff7b7b;
  color: #ff7b7b;
}

.ring strong {
  font-size: 0.75rem;
  color: var(--text-heading);
}
</style>
