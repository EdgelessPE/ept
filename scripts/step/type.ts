export interface StepInfo {
  name: string;
  fields: Array<{
    name: string;
    type: {
      identifier: string;
      optional: boolean;
      enums?: {
        values: string[];
        default?: string;
      };
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
      level: string;
      targets: string;
      scene?: string;
    }>;
  };
}
