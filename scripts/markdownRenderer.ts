import { FieldInfo } from "./type";

// 渲染单一字段
function fieldRenderer(info:FieldInfo,{
  titleLevel
}:{titleLevel:number}):string{
  const clearWiki=info.wiki.replace(/</g,'\\<').replace(/>/g,'\\>')
  return `
${"#".repeat(titleLevel)} ${info.name}
${info.type.optional?"<Tag>可选</Tag> ":""}${clearWiki}
* 类型 \`${info.type.identifier}\``
}

// 渲染一个结构
export function structRenderer(tableName:string,fields:FieldInfo[],{
  titleLevel
}:{titleLevel:number}){
  const needImportTag=fields.find(item=>item.type.optional)
  const fieldsText=fields.map(item=>fieldRenderer(item,{titleLevel:titleLevel+1})).join('')

  return `${"#".repeat(titleLevel)} ${tableName} 
${needImportTag?'\nimport { Tag } from "../../components/tag.tsx"':''}
${fieldsText}`
}