import fs from 'fs'
import { parseFilePath } from './utils'
import { type CommonFieldInfo } from './struct/type'

// 优雅地在 md 行之间加入换行符
function gracefulJoinMarkdown (lines: string[]): string {
  let insideCodeBlock = false
  let finalText = ''

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    if (line.startsWith('```')) {
      insideCodeBlock = !insideCodeBlock
    }
    if (i === lines.length - 1) {
      finalText = finalText + line
      break
    }
    if (insideCodeBlock) {
      finalText = finalText + `${line}\n`
    } else {
      finalText = finalText + `${line}\n\n`
    }
  }

  return finalText
}

// 匹配用花括号包裹的代码块，返回块内的所有行
function matchBlock (startsWith: string, text: string): string[] {
  const lines = text.split('\n')
  // 检索结构体申明开始行号
  let startLineIndex = -1
  lines.find((line, index) => {
    if (line.startsWith(`${startsWith} {`)) {
      startLineIndex = index
      return true
    } else {
      return false
    }
  })
  if (startLineIndex === -1) return []

  // 向下检索申明结束行号
  let endLineIndex = -1
  for (let i = startLineIndex; i < lines.length; i++) {
    const line = lines[i].trim()
    if (line === '}') {
      endLineIndex = i
      break
    }
  }
  if (endLineIndex === -1) return []

  return lines.slice(startLineIndex + 1, endLineIndex)
}

// 匹配代码块并解析注释
export function splitBlock ({
  file,
  startsWith
}: {
  file: string
  startsWith: string
}): CommonFieldInfo[] {
  const filePath = parseFilePath(file)
  if (!fs.existsSync(filePath)) {
    throw new Error(`Error:Failed to open file '${filePath}'`)
  }
  const text = fs.readFileSync(filePath).toString()
  const lines = matchBlock(startsWith, text)
  if (lines.length === 0) {
    throw new Error(
      `Error:Failed to find block starts with '${startsWith}' in '${filePath}'`
    )
  }

  const result: CommonFieldInfo[] = []
  let wikiStack: string[] = []
  let demoStack: string[] = []

  const clearLines = lines.map((line) => line.trim())
  for (const line of clearLines) {
    // 将 wiki 和 demo 注释推入栈
    if (line.startsWith('/// ')) {
      wikiStack.push(line.slice(4))
    }
    if (line.startsWith('//# ')) {
      demoStack.push(line.slice(4))
    }
    // 表示这是一个多行代码块中的空行
    if (line === '//#') {
      demoStack.push('')
    }

    // 忽略普通或其他特殊注释
    if (line.startsWith('//')) continue

    // 走到这个位置说明匹配到申明语句了

    // 特殊处理多行示例代码
    if ((demoStack.length > 0) && demoStack[0].startsWith('```')) {
      demoStack = demoStack.map((line) => `  ${line}`)
      demoStack.unshift('')
    }
    result.push({
      declaration: line,
      wiki: (wikiStack.length > 0) ? gracefulJoinMarkdown(wikiStack) : undefined,
      demo: (demoStack.length > 0) ? demoStack.join('\n') : undefined
    })

    wikiStack = []
    demoStack = []
  }
  return result
}
