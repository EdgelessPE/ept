import { parseStruct } from "./struct";

console.log(parseStruct({
  file:"@/types/package.rs",
  structName:'Package'
}).unwrap());
