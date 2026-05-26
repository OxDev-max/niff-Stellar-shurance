'use client'

import { useCallback, useState } from 'react'

export interface UseCopyToClipboardResult {
  copy: (text: string) => Promise<void>
  copied: boolean
}

export function useCopyToClipboard(resetMs = 2000): UseCopyToClipboardResult {
  const [copied, setCopied] = useState(false)

  const copy = useCallback(
    async (text: string) => {
      try {
        if (navigator.clipboard?.writeText) {
          await navigator.clipboard.writeText(text)
        } else {
          // Fallback for older browsers
          const el = document.createElement('textarea')
          el.value = text
          el.style.position = 'fixed'
          el.style.opacity = '0'
          document.body.appendChild(el)
          el.select()
          document.execCommand('copy')
          document.body.removeChild(el)
        }
        setCopied(true)
        setTimeout(() => setCopied(false), resetMs)
      } catch {
        // copy failed — leave copied as false
      }
    },
    [resetMs],
  )

  return { copy, copied }
}
