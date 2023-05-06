export function* map<T, R>(iter: Iterable<T>, fn: (p: T) => R): Generator<R> {
    let values = iter[Symbol.iterator]()
    let next = values.next()
    while (!next.done) {
        yield fn(next.value)
        next = values.next()
    }
}

export function filter<T>(iter: Iterable<T>, fn: (p: T) => boolean): Generator<T>
export function filter<T, R extends T = T>(iter: Iterable<T>, fn: (p: T) => p is R): Generator<R>
export function* filter<T, R extends T = T>(iter: Iterable<T>, fn: ((p: T) => p is R) | ((p: T) => boolean)): Generator<R | T> {
    let values = iter[Symbol.iterator]()
    let next = values.next()
    while (!next.done) {
        if (fn(next.value)) {
            yield next.value
        }
        next = values.next()
    }
}

export function* unique_by<T, U>(iter: Iterable<T>, fn: (p: T) => U): Generator<T> {
    let values = iter[Symbol.iterator]()
    let seen = new Set<U>()
    let next = values.next()
    while (!next.done) {
        const key = fn(next.value);
        if (!seen.has(key)) {
            yield next.value
            seen.add(key)
        }
        next = values.next()
    }
}

export function* concat<T>(first: Iterable<T>, ...others: Iterable<T>[]): Generator<T> {
    let values = first[Symbol.iterator]()
    let next = values.next()
    let idx = 0;
    while (!next.done || idx < others.length) {
        if (next.done) {
            values = others[idx][Symbol.iterator]()
            idx++;
        } else {
            yield next.value
        }
        next = values.next()
    }
}
