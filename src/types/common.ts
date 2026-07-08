/** 键值对 */
export interface KeyValuePair {
  key: string
  value: string
}

/** 所有测试元素的基类型 */
export interface TestElement {
  id: string
  type: string
  name: string
  enabled: boolean
  comments?: string
}

/**
 * 可包含子元素 (Controller, ThreadGroup)
 */
export interface HasChildren {
  children: TestElement[]
}

/** 请求体类型 */
export type HttpBodyMode = 'none' | 'raw' | 'form-data' | 'x-www-form-urlencoded'

export interface FormDataItem {
  key: string
  value: string
  type: 'text' | 'file'
  filename?: string
  mimeType?: string
}

export interface HttpBody {
  mode: HttpBodyMode
  raw?: string
  contentType?: string
  formData?: FormDataItem[]
  urlEncoded?: KeyValuePair[]
}

/** HTTP 方法 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS'

/** 错误处理策略 */
export type OnErrorAction = 'continue' | 'startNextLoop' | 'stopThread' | 'stopTest'

/** HTTP 认证配置 */
export type AuthType = 'none' | 'basic' | 'bearer'

export interface HttpAuth {
  type: AuthType
  username?: string
  password?: string
  token?: string
}
