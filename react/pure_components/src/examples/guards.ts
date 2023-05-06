export type Guard<T, IS extends T = T> = (p: T) => p is IS

export function guard<T, IS extends T = T>(fn: (p: T) => boolean): Guard<T, IS> {
    return fn as Guard<T, IS>
}
