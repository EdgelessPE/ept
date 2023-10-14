import { parseInnerValues } from './context/values'
import { genStructsWiki } from './struct'

console.log(parseInnerValues('@/executor/values.rs'))

genStructsWiki(
  {
    title: '包描述文件',
    description:
      '描述 Nep 包的基本信息，表位于 [`package.toml`](/nep/struct/2-inner.html#包描述文件)。'
  },
  [
    {
      file: '@/types/package.rs',
      structName: 'Package',
      description: '通用信息表。'
    },
    {
      file: '@/types/software.rs',
      structName: 'Software',
      description: '软件包独占表。'
    }
  ],
  '1-package'
)
