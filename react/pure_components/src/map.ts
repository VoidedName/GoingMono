import {Result} from "./examples/result";
import {Unit} from "./examples/todos/Unit";

export function get<K, V>(map: Map<K, V>, k: K): Result<V, Unit> {
    if (map.has(k)) return Result.ok(map.get(k) as V)
    else return Result.err(Unit)
}

export function delete_of_set<K, V>(map: Map<K, Set<V>>, k: K, v: V) {
    get(map, k)
        .match((set) => {
                set.delete(v)
                if (set.size === 0) {
                    map.delete(k)
                }
            },
            () => {
            })
}

export function add_of_set<K, V>(map: Map<K, Set<V>>, k: K, v: V) {
    get(map, k).match(
        (set) => { set.add(v) },
        () => { map.set(k, new Set([v])) }
    )
}
