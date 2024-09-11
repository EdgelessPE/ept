import * as path from "path";
import { defineConfig } from "rspress/config";
import { pluginShiki } from "@rspress/plugin-shiki";

export default defineConfig({
  root: path.join(__dirname, "doc"),
  // base:'ept',
  title: "ept",
  description: "Next-generation Windows package management solution",
  icon: "https://home.edgeless.top/favicon.ico",
  themeConfig: {
    footer: {
      message: "MIT Licensed | Rendered by Rspress",
    },
    socialLinks: [
      {
        icon: "github",
        mode: "link",
        content: "https://github.com/EdgelessPE/ept",
      },
    ],
    editLink: {
      docRepoBaseUrl: "https://github.com/EdgelessPE/ept/edit/develop/doc",
      text: "Edit this page on GitHub",
    },
    locales: [
      {
        lang: "zh",
        label: "简体中文",
        editLink: {
          docRepoBaseUrl: "https://github.com/EdgelessPE/ept/edit/develop/doc",
          text: "在 GitHub 上编辑此页",
        },
        prevPageText: "上一篇",
        nextPageText: "下一篇",
        outlineTitle: "目录",
        searchPlaceholderText: "搜索",
        searchNoResultsText: "未搜索到相关结果",
        searchSuggestedQueryText: "可更换不同的关键字后重试",
      },
      {
        lang: "en",
        label: "English",
        editLink: {
          docRepoBaseUrl: "https://github.com/EdgelessPE/ept/edit/develop/doc",
          text: "Edit this page on GitHub",
        },
        searchPlaceholderText: "Search",
      },
    ],
  },
  markdown: {
    checkDeadLinks: true,
  },
  plugins: [pluginShiki()],
  lang: "zh",
  locales: [
    {
      lang: "en",
      label: "English",
    },
    {
      lang: "zh",
      label: "简体中文",
    },
  ],
});
