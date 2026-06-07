<template>
  <a-layout class="app-layout">
    <!-- PC 侧边栏 -->
    <a-layout-sider
      v-if="!isMobile"
      width="200"
      :theme="themeStore.isDark ? 'dark' : 'light'"
      class="layout-sider"
    >
      <div class="logo">记账</div>
      <a-menu
        :theme="themeStore.isDark ? 'dark' : 'light'"
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
        <a-menu-item key="/tags">
          <TagOutlined />
          <span>标签</span>
        </a-menu-item>
        <a-menu-item key="/reports">
          <BarChartOutlined />
          <span>报表</span>
        </a-menu-item>
      </a-menu>

      <!-- 当前成员选择器 -->
      <div class="sider-member">
        <a-select
          v-model:value="currentId"
          style="width: 100%"
          size="small"
          :bordered="false"
          :dropdown-match-select-width="false"
          @update:value="handleChange"
        >
          <a-select-option
            v-for="m in memberStore.members"
            :key="m.id"
            :value="m.id"
          >
            {{ m.name }}
          </a-select-option>
          <template #dropdownRender="{ menuNode: menu }">
            <div>
              <component :is="menu" />
              <a-divider style="margin: 4px 0" />
              <div
                v-if="!adding"
                style="padding: 4px 12px; cursor: pointer"
                @mousedown.prevent
                @click="adding = true"
              >
                <PlusOutlined />
                <span style="margin-left: 4px">添加成员</span>
              </div>
              <div
                v-else
                style="padding: 4px 12px; display: flex; gap: 4px"
              >
                <a-input
                  v-model:value="newName"
                  ref="addInputRef"
                  size="small"
                  placeholder="成员名"
                  style="flex: 1"
                  @press-enter="handleAdd"
                />
                <a-button size="small" type="primary" @click="handleAdd">
                  确认
                </a-button>
              </div>
            </div>
          </template>
        </a-select>
      </div>

      <!-- 设置按钮 -->
      <div class="sider-settings">
        <a-button type="text" block @click="drawerOpen = true">
          <SettingOutlined />
          <span style="margin-left: 4px">设置</span>
        </a-button>
      </div>
    </a-layout-sider>

    <!-- 内容区 -->
    <a-layout-content
      class="layout-content"
      :class="{ 'mobile-content': isMobile }"
    >
      <!-- 移动端顶部栏 -->
      <div v-if="isMobile" class="mobile-top-bar">
        <div class="mobile-member">
          <span class="member-label">当前成员</span>
          <a-select
            v-model:value="currentId"
            style="width: 120px"
            size="small"
            :bordered="false"
            :dropdown-match-select-width="false"
            @update:value="handleChange"
          >
            <a-select-option
              v-for="m in memberStore.members"
              :key="m.id"
              :value="m.id"
            >
              {{ m.name }}
            </a-select-option>
            <template #dropdownRender="{ menuNode: menu }">
              <div>
                <component :is="menu" />
                <a-divider style="margin: 4px 0" />
                <div
                  v-if="!adding"
                  style="padding: 4px 12px; cursor: pointer"
                  @mousedown.prevent
                  @click="adding = true"
                >
                  <PlusOutlined />
                  <span style="margin-left: 4px">添加成员</span>
                </div>
                <div
                  v-else
                  style="padding: 4px 12px; display: flex; gap: 4px"
                >
                  <a-input
                    v-model:value="newName"
                    ref="addInputRef"
                    size="small"
                    placeholder="成员名"
                    style="flex: 1"
                    @press-enter="handleAdd"
                  />
                  <a-button size="small" type="primary" @click="handleAdd">
                    确认
                  </a-button>
                </div>
              </div>
            </template>
          </a-select>
        </div>
        <a-button type="text" size="small" @click="drawerOpen = true">
          <SettingOutlined />
        </a-button>
      </div>
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

    <!-- 设置抽屉 -->
    <a-drawer
      v-model:open="drawerOpen"
      title="设置"
      placement="right"
      :width="280"
    >
      <div class="drawer-section">
        <h4>外观</h4>
        <a-radio-group v-model:value="themeStore.theme" @change="handleThemeChange">
          <a-radio-button value="auto">跟随系统</a-radio-button>
          <a-radio-button value="light">明亮</a-radio-button>
          <a-radio-button value="dark">暗色</a-radio-button>
        </a-radio-group>
      </div>
    </a-drawer>
  </a-layout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  HomeOutlined,
  BookOutlined,
  TagOutlined,
  BarChartOutlined,
  PlusOutlined,
  SettingOutlined,
} from '@ant-design/icons-vue'
import { useMemberStore } from '@/stores/member'
import { useThemeStore } from '@/stores/theme'
import api from '@/api/client'

const route = useRoute()
const router = useRouter()
const memberStore = useMemberStore()
const themeStore = useThemeStore()
const isMobile = ref(window.innerWidth < 768)
const adding = ref(false)
const newName = ref('')
const addInputRef = ref<HTMLInputElement | null>(null)
const drawerOpen = ref(false)

const currentRoute = computed(() => route.path)
const currentId = computed({
  get: () => memberStore.currentMember?.id ?? undefined,
  set: () => {},
})

const tabs = [
  { path: '/', label: '首页', icon: HomeOutlined },
  { path: '/accounts', label: '账户', icon: BookOutlined },
  { path: '/tags', label: '标签', icon: TagOutlined },
  { path: '/reports', label: '报表', icon: BarChartOutlined },
]

function handleMenuClick(info: { key: string | number }) {
  router.push(String(info.key))
}

function handleResize() {
  isMobile.value = window.innerWidth < 768
}

function handleThemeChange(e: any) {
  const val = e?.target?.value ?? themeStore.theme
  themeStore.setTheme(val)
}

async function handleChange(value: number) {
  await memberStore.setCurrent(value)
}

async function handleAdd() {
  const name = newName.value.trim()
  if (!name) {
    adding.value = false
    return
  }
  try {
    await api.post('/members', { name })
    await memberStore.fetchMembers()
    adding.value = false
    newName.value = ''
  } catch (e) {
    console.error('创建成员失败', e)
  }
}

watch(adding, (val) => {
  if (val) {
    nextTick(() => {
      addInputRef.value?.focus()
    })
  }
})

onMounted(() => {
  window.addEventListener('resize', handleResize)
  memberStore.fetchMembers()
  memberStore.fetchMe()
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
  color: #333;
  font-size: 20px;
  font-weight: bold;
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
}

.layout-sider.ant-layout-sider-dark .logo {
  color: #fff;
  border-bottom-color: rgba(255, 255, 255, 0.1);
}

.layout-content {
  padding: 24px;
  background: #f0f2f5;
}

.mobile-content {
  padding: 16px 16px 80px;
}

.sider-member {
  position: absolute;
  bottom: 48px;
  left: 16px;
  right: 16px;
}

.layout-sider.ant-layout-sider-dark .sider-member :deep(.ant-select-selector) {
  background: rgba(255, 255, 255, 0.1) !important;
  color: #fff !important;
}

.layout-sider.ant-layout-sider-dark .sider-member :deep(.ant-select-selection-item) {
  color: #fff !important;
}

.layout-sider.ant-layout-sider-dark .sider-member :deep(.ant-select-arrow) {
  color: rgba(255, 255, 255, 0.6) !important;
}

.sider-settings {
  position: absolute;
  bottom: 8px;
  left: 8px;
  right: 8px;
}

.layout-sider.ant-layout-sider-dark .sider-settings :deep(.ant-btn) {
  color: rgba(255, 255, 255, 0.65);
}

.layout-sider.ant-layout-sider-dark .sider-settings :deep(.ant-btn:hover) {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}

.mobile-top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: #fff;
  border-bottom: 1px solid #f0f0f0;
  margin: -16px -16px 12px;
}

.mobile-member {
  display: flex;
  align-items: center;
  gap: 8px;
}

.member-label {
  font-size: 13px;
  color: #666;
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

.drawer-section h4 {
  margin-bottom: 12px;
  font-size: 14px;
  color: #666;
}
</style>
