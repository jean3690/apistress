<script setup lang="ts">
import { useUIStore } from '@/stores'
import ToolBar from './ToolBar.vue'
import StatusBar from './StatusBar.vue'
import ClassicLayout from '@/components/classic/ClassicLayout.vue'
import ModernLayout from '@/components/modern/ModernLayout.vue'

const ui = useUIStore()
</script>

<template>
  <div class="app-shell">
    <ToolBar />
    <div class="main-area">
      <ClassicLayout v-if="ui.mode === 'classic'" />
      <ModernLayout v-else />
    </div>
    <StatusBar />
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.main-area {
  flex: 1;
  overflow: hidden;
  display: flex;
}
</style>
