import * as path from 'path';
import { defineConfig } from 'rspress/config';

export default defineConfig({
  root: path.join(__dirname, 'doc'),
  // base:'ept',
  title:'ept',
  description:'Modern package solution for Windows',
  icon:'https://home.edgeless.top/favicon.ico',
  themeConfig: {
    footer:{
      message:'MPL-2.0 Licensed | Rendered by Rspress',
    },
    socialLinks: [
      {
        icon: 'github',
        mode:'link',
        content: 'https://github.com/EdgelessPE/ept',
      },
    ],
    editLink: {
      docRepoBaseUrl:
        'https://github.com/EdgelessPE/ept/edit/develop/doc',
      text: 'Edit this page on GitHub',
    },
  },
  markdown:{
    checkDeadLinks: true,
  },
});
