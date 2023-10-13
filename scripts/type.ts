export interface FieldInfo {
  name: string;
  type: {
    identifier: string;
    optional: boolean;
  };
  wiki: string;
}

export interface FileInfo {
  file: string;
  structName: string;
  description?: string;
}
