import { ReactNode } from "react"

export const Tag=({children}:{children:ReactNode})=>{
  return <div style={{
    display:'inline-block',
    border:'1px solid transparent',
    borderRadius: 10,
    color:'var(--rp-c-brand-dark)',
    background:'#eaf3fe',
    padding:'0 8px'
  }}>{children}</div>
}