import { genStructsWiki } from "./struct";
import { genContextWiki } from "./context";
import { genPermissionsWiki } from "./permission";
import { genStepsWiki } from "./step";
import { genWorkflowWiki } from "./workflow";

genStructsWiki(
  {
    title: "包描述文件",
    description:
      "描述 Nep 包的基本信息，表位于 [`package.toml`](/nep/struct/2-inner.html#包描述文件)。",
  },
  [
    {
      file: "@/types/package.rs",
      structName: "Package",
      description: "通用信息表。",
    },
    {
      file: "@/types/software.rs",
      structName: "Software",
      description: "软件包独占表。",
    },
  ],
  "1-package",
);

genContextWiki(
  {
    top: {
      title: "上下文",
      description:
        "提供在工作流执行过程中可用的上下文信息，例如内置变量和内置函数。",
    },
    valuesTop: {
      title: "内置变量",
      description:
        "步骤字段中可用的内置变量。注意在非条件字段中使用时需要使用模板写法，详见[内置变量](/nep/workflow/2-context.html#内置变量)。\n\n“权限等级”字段表示该内置变量对应路径在访问时所需要的[权限](/nep/ability/1-permission)等级。",
    },
    fnTop: {
      title: "内置函数",
      description:
        "步骤的条件语句可用的内置变量。当前版本提供的内置变量都是输入为`String`输出为`Bool`的简单函数。",
    },
  },
  {
    valuesFile: "@/executor/values.rs",
    fnDir: "@/executor/functions",
    appendValues: [
      {
        name: "ExitCode",
        level: "Normal",
        wiki: "上一步骤的退出码，**类型为整数**。若步骤被正常执行则其值为 0，否则不为 0。",
        demoValue: 0,
      },
      {
        name: "DefaultLocation",
        level: "Normal",
        wiki: "当前包的默认安装位置",
        demo: `to = "\${DefaultLocation}/config"`,
        demoValue: "C:/Users/UserName/ept/Microsoft/VSCode",
      },
    ],
  },
  "2-context",
);

genPermissionsWiki({
  file: "@/types/permissions.rs",
  top: {
    title: "权限",
    description:
      "Nep 规范支持 ept 通过对包的工作流上下文进行总结获得该包生命周期过程中需要使用的权限信息，便于用户在确认使用该包之前查看所有的权限信息。",
  },
  levelTop: {
    title: "权限等级",
    description:
      "不同类型、不同目标的权限会被分为不同的权限等级，下列排序为权限敏感程度升序排列。",
  },
  keyTop: {
    title: "权限类型",
    description:
      "不同的工作流步骤或内置函数调用会产生不同类型的权限。请注意有些时候即使权限类型相同，其对应的权限等级也有可能不同。",
  },
  toFileName: "3-permissions",
});

genStepsWiki(
  {
    title: "步骤",
    description: "通过步骤组成工作流完成指定操作。",
  },
  {
    srcDir: "@/types/steps",
  },
  "4-steps",
);
genWorkflowWiki({
  file: "@/types/workflow.rs",
  top: {
    title: "工作流",
    description: "在步骤上附加的公共工作流字段定义。",
  },
  titleLevel: 1,
});
