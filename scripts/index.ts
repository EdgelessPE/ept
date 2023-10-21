import { genStructsWiki } from "./struct";
import { genContext } from "./context";

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

genContext(
  {
    top: {
      title: "上下文",
      description:
        "提供在工作流执行过程中可用的上下文信息，例如内置变量和内置函数。",
    },
    valuesTop: {
      title: "内置变量",
      description:
        "步骤字段中可用的内置变量。注意在非条件字段中使用时需要使用模板写法，详见[内置变量](/nep/workflow/2-context.html#内置变量)。",
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
        wiki: "上一步骤的退出码，**类型为整数**。",
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
