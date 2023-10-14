import fs from 'fs'
import path from 'path'

export function writeWiki (
  {
    title,
    imports,
    description,
    content
  }: {
    title: string
    imports?: string[]
    description?: string
    content: string
  },
  toFileName: string
) {
  const importsText = imports ? `\n${imports.join('\n')}\n` : ''
  const descriptionText = description ? `\n${description}` : ''

  const finalText = `# ${title}

{/* This file is automatically generated by script, do not modify it. */}
${importsText} ${descriptionText} \n${content}`

  fs.writeFileSync(
    path.join(__dirname, `../doc/nep/definition/${toFileName}.mdx`),
    finalText
  )
}
