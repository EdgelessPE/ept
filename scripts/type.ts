export interface FieldInfo {
  name: string
  type: {
    identifier: string
    optional: boolean
    enum?: string[]
  }
  wiki?: string
  demo?: string
}

export interface CommonFieldInfo {
  wiki?: string
  demo?: string
  declaration: string
}

export interface FileInfo {
  file: string
  structName: string
  description?: string
}

export type PermissionLevel = 'Normal' | 'Important' | 'Sensitive'
