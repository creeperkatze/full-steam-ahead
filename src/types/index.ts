import type { ImportSource } from '../bindings'

export type * from '../bindings'

export type ScannableSource = Exclude<Extract<ImportSource, string>, 'manual'>
