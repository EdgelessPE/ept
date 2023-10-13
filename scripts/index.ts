import { structRenderer } from "./markdownRenderer";
import { parseStruct } from "./struct";
import fs from 'fs'

const fields=parseStruct({
  file:"@/types/package.rs",
  structName:'Package'
}).unwrap()
const text=structRenderer('package',fields,{titleLevel:1});
fs.writeFileSync('/Users/bytedance/Projects/ept/doc/nep/ability/1-permission.mdx',text)