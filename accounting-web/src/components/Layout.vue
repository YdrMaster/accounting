<template>
  <a-layout class="app-layout">
    <!-- PC 侧边栏 -->
    <a-layout-sider
      v-if="!isMobile"
      width="200"
      theme="dark"
      class="layout-sider"
    >
      <div class="logo">记账</div>
      <a-menu
        theme="dark"
        mode="inline"
        :selected-keys="[currentRoute]"
        @click="handleMenuClick"
      >
        <a-menu-item key="/">
          <HomeOutlined />
          <span>首页</span>
        </a-menu-item>
        <a-menu-item key="/accounts">
          <BookOutlined />
          <span>账户</span>
        </a-menu-item>
        <a-menu-item key="/reports">
          <BarChartOutlined />
          <span>报表</span>
        </a-menu-item>
        <a-menu-item key="/settings">
          <SettingOutlined />
          <span>设置</span>
        </a-menu-item>
      </a-menu>
    </a-layout-sider>

    <!-- 内容区 -->
    <a-layout-content
      class="layout-content"
      :class="{ 'mobile-content': isMobile }"
    >
      <router-view />
    </a-layout-content>

    <!-- 手机端底部 Tab -->
    <div v-if="isMobile" class="mobile-tab-bar">
      <div
        v-for="tab in tabs"
        :key="tab.path"
        class="tab-item"
        :class="{ active: currentRoute === tab.path }"
        @click="router.push(tab.path)"
      >
        <component :is="tab.icon" />
        <span>{{ tab.label }}</span>
      </div>
    </div>
  </a-layout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  HomeOutlined,
  BookOutlined,
  BarChartOutlined,
  SettingOutlined,
} from '@ant-design/icons-vue'

const route = useRoute()
const router = useRouter()
const isMobile = ref(window.innerWidth < 768)

const currentRoute = computed(() => route.path)

const tabs = [
  { path: '/', label: '首页', icon: HomeOutlined },
  { path: '/accounts', label: '账户', icon: BookOutlined },
  { path: '/reports', label: '报表', icon: BarChartOutlined },
  { path: '/settings', label: '设置', icon: SettingOutlined },
]

function handleMenuClick(info: { key: string | number }) {
  router.push(String(info.key))
}

function handleResize() {
  isMobile.value = window.innerWidth < 768
}

onMounted(() => {
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
})
</script>

<style scoped>
.app-layout {
  min-height: 100vh;
}

.logo {
  height: 64px;
  line-height: 64px;
  text-align: center;
  color: #fff;
  font-size: 20px;
  font-weight: bold;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.layout-content {
  padding: 24px;
  background: #f0f2f5;
}

.mobile-content {
  padding: 16px 16px 80px;
}

.mobile-tab-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 56px;
  background: #fff;
  border-top: 1px solid #e8e8e8;
  display: flex;
  align-items: center;
  justify-content: space-around;
  z-index: 100;
}

.tab-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #999;
  font-size: 12px;
  cursor: pointer;
  flex: 1;
  height: 100%;
}

.tab-item.active {
  color: #1890ff;
}
</style>
