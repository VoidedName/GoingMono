type ParameterToFn<P, R> = (p: P) => R

type TupleToFn<T, R> = T extends []
    ? []
    : T extends [first: infer A, ...rest: infer Rest]
        ? [ParameterToFn<A, R>, ...TupleToFn<Rest, R>]
        : never

export type PatternMatch<T extends any[]> = {
    match: <V, >(...branches: TupleToFn<T, V>) => V
}
