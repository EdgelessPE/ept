import fs from "fs";
import path from "path";
import { Result, Ok, Err } from "ts-results";
import { FieldInfo, FileInfo } from "./type";
import { parseFilePath } from "./utils";
import { parseEnumDefinitions } from "./enum";

// 给定源文件内容，选择一个结构体名称对应的代码块
function matchStructBlock(name:string,text:string):string[]{
  const lines=text.split('\n')
  // 检索结构体申明开始行号
  let startLineIndex=-1;
  lines.find((line,index)=>{
    if(line.startsWith(`pub struct ${name} {`)){
      startLineIndex=index
      return true
    }else{
      return false
    }
  })
  if(startLineIndex===-1) return []

  // 向下检索申明结束行号
  let endLineIndex=-1
  for(let i=startLineIndex;i<lines.length;i++){
    const line=lines[i]
    if(line==="}"){
      endLineIndex=i
      break
    }
  }
  if(endLineIndex===-1) return []
  
  return lines.slice(startLineIndex+1,endLineIndex)
}

// 读取 Rust 中的某个 struct，分析出所有字段信息
export function parseStruct(fileInfo: FileInfo): Result<FieldInfo[], string> {
  let { file, structName } = fileInfo;

  // 解析路径
  file = parseFilePath(file);

  // 打开文件
  if (!fs.existsSync(file)) {
    return new Err(`Error:Failed to open file '${file}'`);
  }
  const text = fs.readFileSync(file).toString();

  // 解析枚举值
  const enumValuesMap = parseEnumDefinitions(fileInfo);

  // 匹配结构体
  const m = matchStructBlock(structName,text);
  if (!m.length) {
    return new Err(
      `Error:Failed to find struct '${structName}' in file '${file}'`
    );
  }

  // 清理数据并按行分割
  const clearMatches = m
    .map((line) => {
      let r = line.trim();
      if (r.startsWith("pub ")) {
        r = r.slice(4);
      }
      return r;
    });

  // 解析
  const result: FieldInfo[] = [];
  let wikiStack: string[] = [];
  let demoStack: string[] = [];
  for (const line of clearMatches) {
    // 将 wiki 和 demo 注释推入栈
    if (line.startsWith("/// ")) {
      wikiStack.push(line.slice(4));
    }
    if (line.startsWith("//# ")) {
      demoStack.push(line.slice(4));
    }
    // 表示这是一个多行代码块中的空行
    if (line==="//#") {
      demoStack.push('');
    }

    // 忽略普通或其他特殊注释
    if (line.startsWith("//")) continue;

    // 解析字段名和类型
    const m = line.match(/(\w+):\s?([\w<>()]+)/);
    if (m) {
      const [, name, rawType] = m;
      const enumValues = enumValuesMap[name];
      const type: FieldInfo["type"] =
        rawType.startsWith("Option<") && rawType.endsWith(">")
          ? {
              identifier: rawType.slice(7, -1),
              optional: true,
              enum: enumValues,
            }
          : {
              identifier: rawType,
              optional: false,
              enum: enumValues,
            };
      if (enumValues) {
        if (type.identifier !== "String") {
          throw new Error(
            `Error:Field '${name}' has enum but not a string (got '${type.identifier}')`
          );
        } else {
          type.identifier = "String 枚举";
        }
      }
      // 特殊处理多行示例代码
      if(demoStack.length&&demoStack[0].startsWith('```')){
        demoStack=demoStack.map(line=>`  ${line}`)
        demoStack.unshift("")
      }
      result.push({
        name,
        type,
        wiki: wikiStack.length ? wikiStack.join("\n\n") : undefined,
        demo: demoStack.length ? demoStack.join("\n") : undefined,
      });
      wikiStack = [];
      demoStack = [];
    } else {
      return new Err(
        `Error:Failed to parse line '${line}' as valid rust field declaration`
      );
    }
  }

  return new Ok(result);
}
