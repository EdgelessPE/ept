import path from 'path'

export function parseFilePath (rawPath: string) {
  if (rawPath.startsWith('@/')) {
    rawPath = rawPath.replace('@/', path.join(__dirname, '../src/'))
  }
  return rawPath
}

// 优雅地在 md 行之间加入换行符
export function gracefulJoinMarkdown (lines: string[]): string {
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
