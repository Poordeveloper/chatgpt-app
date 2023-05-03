<script setup lang='ts'>
import { defineAsyncComponent, onMounted, ref } from 'vue'
import { HoverButton, SvgIcon, UserAvatar } from '@/components/common'
import { get, isTauri } from '@/tauri'

const Setting = defineAsyncComponent(() => import('@/components/common/Setting/index.vue'))

const show = ref(false)
const showTab = ref('General')

async function checkApiKey() {
  if (!isTauri)
    return
  try {
    show.value = !(await get('OPENAI_API_KEY'))
    if (show.value)
      showTab.value = 'Config'
  }
  finally {
    //
  }
}

onMounted(() => {
  checkApiKey()
})
</script>

<template>
  <footer class="flex items-center justify-between min-w-0 p-4 overflow-hidden border-t dark:border-neutral-800">
    <div class="flex-1 flex-shrink-0 overflow-hidden">
      <UserAvatar />
    </div>

    <HoverButton @click="show = true">
      <span class="text-xl text-[#4f555e] dark:text-white">
        <SvgIcon icon="ri:settings-4-line" />
      </span>
    </HoverButton>

    <Setting v-if="show" v-model:visible="show" v-model:active="showTab" />
  </footer>
</template>
