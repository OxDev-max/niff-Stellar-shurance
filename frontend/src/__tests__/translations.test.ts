/**
 * @jest-environment node
 *
 * Verifies that the Spanish (es) locale has no missing keys relative to the
 * English (en) base locale. This mirrors the CI check in
 * scripts/check-translations.js but runs inside Jest so failures are surfaced
 * in the standard test report.
 */

import fs from 'fs'
import path from 'path'

const MESSAGES_DIR = path.resolve(__dirname, '../../messages')
const BASE_LOCALE = 'en'
const CATALOGS = ['common', 'policy', 'claims', 'wallet'] as const

/** Flatten a nested object to dot-notation keys, e.g. { a: { b: 1 } } → { 'a.b': 1 } */
function flatten(obj: Record<string, unknown>, prefix = ''): Record<string, unknown> {
  return Object.entries(obj).reduce<Record<string, unknown>>((acc, [k, v]) => {
    const key = prefix ? `${prefix}.${k}` : k
    if (v !== null && typeof v === 'object' && !Array.isArray(v)) {
      Object.assign(acc, flatten(v as Record<string, unknown>, key))
    } else {
      acc[key] = v
    }
    return acc
  }, {})
}

function loadCatalog(locale: string, catalog: string): Record<string, unknown> | null {
  const filePath = path.join(MESSAGES_DIR, locale, `${catalog}.json`)
  if (!fs.existsSync(filePath)) return null
  return JSON.parse(fs.readFileSync(filePath, 'utf8')) as Record<string, unknown>
}

const locales = fs
  .readdirSync(MESSAGES_DIR)
  .filter((d) => fs.statSync(path.join(MESSAGES_DIR, d)).isDirectory())
  .filter((d) => d !== BASE_LOCALE)

describe('i18n translation completeness', () => {
  for (const catalog of CATALOGS) {
    describe(`catalog: ${catalog}`, () => {
      const baseData = loadCatalog(BASE_LOCALE, catalog)

      it(`en/${catalog}.json exists`, () => {
        expect(baseData).not.toBeNull()
      })

      if (!baseData) return

      const baseKeys = Object.keys(flatten(baseData))

      for (const locale of locales) {
        describe(`locale: ${locale}`, () => {
          it(`${locale}/${catalog}.json exists`, () => {
            const data = loadCatalog(locale, catalog)
            expect(data).not.toBeNull()
          })

          it(`${locale}/${catalog}.json has no missing keys`, () => {
            const data = loadCatalog(locale, catalog)
            if (!data) {
              // File missing — already caught by the existence test above
              return
            }
            const localeKeys = new Set(Object.keys(flatten(data)))
            const missingKeys = baseKeys.filter((k) => !localeKeys.has(k))

            expect(missingKeys).toEqual([])
          })

          it(`${locale}/${catalog}.json has no undefined values at runtime`, () => {
            const data = loadCatalog(locale, catalog)
            if (!data) return

            const flatData = flatten(data)
            const undefinedKeys = Object.entries(flatData)
              .filter(([, v]) => v === undefined || v === null || v === '')
              .map(([k]) => k)

            expect(undefinedKeys).toEqual([])
          })
        })
      }
    })
  }
})
