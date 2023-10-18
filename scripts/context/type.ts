export type PermissionLevel = "Normal" | "Important" | "Sensitive";
export interface ValueInfo {
  name: string;
  level: PermissionLevel;
  wiki?: string;
  demo?: string;
  demoValue?: string | number;
}

export interface FnValue {
  name: string;
  wiki?: string;
  demo?: string;
  permission: {
    key: string;
    level: PermissionLevel | "JUDGE_WITH_PATH";
  };
  validationRules?: string;
}
