import type { PermissionLevel } from "../context/type";

export interface StepInfo {
  name: string;
  fields: Array<{
    name: string;
    type: {
      identifier: string;
      optional: boolean;
    };
    wiki?: string;
    demo?: string;
    rules?: string[];
  }>;
  extra: {
    run: string;
    reverseRun?: string;
    manifest?: string[];
    permissions?: Array<{
      key: string;
      level: PermissionLevel;
      targets: string[];
      scene?: string;
    }>;
  };
}
