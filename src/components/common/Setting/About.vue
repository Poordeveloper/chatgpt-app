<script setup lang='ts'>
import { computed, onMounted, ref } from 'vue'
import { NButton, NInput, NSpin, useMessage } from 'naive-ui'
import { fetchChatConfig } from '@/api'
import pkg from '@/../package.json'
import { useAuthStore } from '@/store'
import { isTauri, set } from '@/tauri'
import { t } from '@/locales'

interface ConfigState {
  apiKey?: string
  timeoutMs?: number
  reverseProxy?: string
  proxy?: string
  usage?: string
}

const authStore = useAuthStore()

const loading = ref(false)

const config = ref<ConfigState>({})

const isChatGPTAPI = computed<boolean>(() => !!authStore.isChatGPTAPI)

const ms = useMessage()

async function fetchConfig() {
  try {
    loading.value = true
    const { data } = await fetchChatConfig<ConfigState>()
    config.value = data
  }
  finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchConfig()
})

function update(key: string, value: string) {
  set(key, value)
  ms.success(t('common.success'))
}
</script>

<template>
  <NSpin :show="loading">
    <div v-if="!isTauri" class="p-4 space-y-4">
      <h2 class="text-xl font-bold">
        Version - {{ pkg.version }}
      </h2>
      <p v-if="isChatGPTAPI">
        {{ $t("setting.monthlyUsage") }}：{{ config.usage || '-' }}
      </p>
      <p v-if="!isChatGPTAPI">
        {{ $t("setting.reverseProxy") }}：{{ config.reverseProxy || '-' }}
      </p>
      <p>{{ $t("setting.timeout") }}：{{ config.timeoutMs || '-' }}</p>
      <p>{{ $t("setting.proxy") }}：{{ config.proxy || '-' }}</p>
    </div>
    <div v-else class="p-4 space-y-5 min-h-[200px]">
      <div class="flex items-center space-x-4">
        <span class="flex-shrink-0 w-[120px]">{{ $t('setting.key') }} </span>
        <div class="flex-1">
          <NInput v-model:value="config.apiKey" type="password" autofocus placeholder="" />
        </div>
        <NButton size="tiny" text type="primary" @click="update('OPENAI_API_KEY', config.apiKey || '')">
          {{ $t('common.save') }}
        </NButton>
      </div>
      <div class="flex items-center space-x-4">
        <span class="flex-shrink-0 w-[120px]">{{ $t('setting.reverseProxy') }} </span>
        <div class="flex-1">
          <NInput v-model:value="config.reverseProxy" placeholder="" />
        </div>
        <NButton size="tiny" text type="primary" @click="update('API_REVERSE_PROXY', config.reverseProxy || '')">
          {{ $t('common.save') }}
        </NButton>
      </div>
      <div class="flex items-center space-x-4">
        <span class="flex-shrink-0 w-[120px]">{{ $t('setting.proxy') }} </span>
        <div class="flex-1">
          <NInput v-model:value="config.proxy" placeholder="[socks5|https]://[<username>:<password>@]<host>[:<port>]" />
        </div>
        <NButton size="tiny" text type="primary" @click="update('PROXY', config.proxy || '')">
          {{ $t('common.save') }}
        </NButton>
      </div>
      <div class="flex items-center space-x-4">
        <span class="flex-shrink-0 w-[120px]">{{ $t('setting.timeout') }} </span>
        <div class="flex-1">
          <NInput v-model:value="config.timeoutMs" type="digit" placeholder="In milliseconds" />
        </div>
        <NButton size="tiny" text type="primary" @click="update('TIMEOUT_MS', String(config.timeoutMs || 0))">
          {{ $t('common.save') }}
        </NButton>
      </div>
      <div v-if="false" class="flex items-center space-x-4">
        <span class="flex-shrink-0 w-[120px]">{{ $t('setting.monthlyUsage') }} </span>
        <div class="flex-1">
          {{ config.usage }}
        </div>
      </div>
    </div>
  </NSpin>
</template>
