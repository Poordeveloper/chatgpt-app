import type { AxiosProgressEvent, AxiosResponse, GenericAbortSignal } from 'axios'
import request from './axios'
import { useAuthStore } from '@/store'
import { call, isTauri } from '@/tauri'

interface HttpOption {
  url: string
  data?: any
  method?: string
  headers?: any
  onDownloadProgress?: (progressEvent: AxiosProgressEvent) => void
  signal?: GenericAbortSignal
  beforeRequest?: () => void
  afterRequest?: () => void
}

export interface Response<T = any> {
  data: T
  message: string | null
  status: string
}

let onProgressId = 0

function http<T = any>(
  { url, data, method, headers, onDownloadProgress, signal, beforeRequest, afterRequest }: HttpOption,
) {
  const successHandler = (res: AxiosResponse<Response<T>>) => {
    const authStore = useAuthStore()

    if (res.data.status === 'Success' || typeof res.data === 'string')
      return res.data

    if (res.data.status === 'Unauthorized') {
      authStore.removeToken()
      window.location.reload()
    }

    return Promise.reject(res.data)
  }

  const failHandler = (error: Response<Error>) => {
    afterRequest?.()
    const msg = isTauri ? error : error?.message
    throw new Error(msg as string || 'Error')
  }

  beforeRequest?.()

  method = method || 'GET'

  const params = Object.assign(typeof data === 'function' ? data() : data ?? {}, {})

  if (isTauri) {
    let on_progress_id: string | undefined
    if (onDownloadProgress) {
      on_progress_id = String(onProgressId)
      onProgressId += 1
      if (!window.on_progress)
        window.on_progress = {}

      window.on_progress[on_progress_id] = onDownloadProgress
    }
    else {
      on_progress_id = undefined
    }
    return call(url, params, on_progress_id).then((arg) => {
      if (on_progress_id)
        delete window.on_progress[on_progress_id]
      return successHandler(arg)
    }, (arg) => {
      if (on_progress_id)
        delete window.on_progress[on_progress_id]
      return failHandler(arg)
    })
  }

  return method === 'GET'
    ? request.get(url, { params, signal, onDownloadProgress }).then(successHandler, failHandler)
    : request.post(url, params, { headers, signal, onDownloadProgress }).then(successHandler, failHandler)
}

export function post<T = any>(
  { url, data, method = 'POST', headers, onDownloadProgress, signal, beforeRequest, afterRequest }: HttpOption,
): Promise<Response<T>> {
  return http<T>({
    url,
    method,
    data,
    headers,
    onDownloadProgress,
    signal,
    beforeRequest,
    afterRequest,
  })
}

export default post
