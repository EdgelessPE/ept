export type PermissionLevel = "Normal" | "Important" | "Sensitive";
export interface ValueInfo {
  name: string;
  level: PermissionLevel;
  wiki?: string;
  demo?: string;
}
