import {Result} from "./result";
import {Unit} from "./todos/Unit";
import {get} from "../map";

export class BiDirectionalMap<Key, Value> {
    private forward = new Map<Key, Value>
    private backward = new Map<Value, Set<Key>>


    get(k: Key): Result<Value, Unit> {
        return get(this.forward, k)
    }

    get_rev(v: Value): Result<Set<Key>, Unit> {
        return get(this.backward, v)
    }

    set(k: Key, v: Value) {
        this.delete(k)
        this.forward.set(k, v)

        get(this.backward, v)
            .match(
                (keys) => { keys.add(k) },
                () => { this.backward.set(v, new Set([k])) }
            )
    }

    delete(k: Key) {
        get(this.forward, k)
            .match(
                (v) => {
                    this.forward.delete(k)
                    let keys = this.backward.get(v)!
                    keys.delete(k)
                    if (keys.size === 0) this.backward.delete(v)
                },
                () => {}
            )
    }
}
