import {PatternMatch} from "./PatternMatch";

abstract class ResultMethods<Ok, Err> implements PatternMatch<[Ok, Err]> {
    is_ok(this: Result<Ok, Err>): this is OK<Ok, Err> {
        return this instanceof OK
    }

    is_ok_and(this: Result<Ok, Err>, predicate: (value: Ok) => boolean): this is OK<Ok, Err> {
        return this instanceof OK && predicate(this.value)
    }

    is_err(this: Result<Ok, Err>): this is ERR<Ok, Err> {
        return this instanceof ERR
    }

    is_err_and(this: Result<Ok, Err>, predicate: (value: Err) => boolean): this is ERR<Ok, Err> {
        return this instanceof ERR && predicate(this.error)
    }

    abstract map<T>(this: Result<Ok, Err>, fn: (value: Ok) => T): Result<T, Err>

    abstract or<F>(this: Result<Ok, Err>, res: Result<Ok, F>): Result<Ok, F>

    abstract or_else<F>(this: Result<Ok, Err>, fallback: (err: Err) => Result<Ok, F>): Result<Ok, F>

    abstract and<O>(this: Result<Ok, Err>, res: Result<O, Err>): Result<O, Err>

    abstract and_then<O>(this: Result<Ok, Err>, op: (value: Ok) => Result<O, Err>): Result<O, Err>

    abstract map_err<T>(this: Result<Ok, Err>, fn: (err: Err) => T): Result<Ok, T>

    inspect(this: Result<Ok, Err>, fn: (v: Ok) => void): Result<Ok, Err> {
        if (this.is_ok()) fn(this.value);
        return this;
    }

    inspect_err(this: Result<Ok, Err>, fn: (e: Err) => void): Result<Ok, Err> {
        if (this.is_err()) fn(this.error);
        return this;
    }

    // Result.iter equivalent, probably useless in that case
    [Symbol.iterator] = function* (this: Result<Ok, Err>): Generator<Ok> {
        if (this.is_ok()) yield this.value;
    }

    /** can throw */
    abstract expect(this: Result<Ok, Err>, error: string): Ok

    /** can throw */
    abstract unwrap(this: Result<Ok, Err>): Ok

    unwrap_or(this: Result<Ok, Err>, fallback: Ok): Ok {
        if (this.is_ok()) return this.value
        else return fallback;
    }

    unwrap_or_else(this: Result<Ok, Err>, fn: (e: Err) => Ok): Ok {
        if (this.is_ok()) return this.value
        else return fn(this.error);
    }

    contains(this: Result<Ok, Err>, value: Ok): this is OK<Ok, Err> {
        return this.is_ok_and((v) => v === value);
    }

    contains_err(this: Result<Ok, Err>, error: Err): this is ERR<Ok, Err> {
        return this.is_err_and((e) => e === error);
    }

    flatten<T>(this: Result<Result<T, Err>, Err>): Result<T, Err> {
        if (this.is_ok()) return this.value
        else return this as unknown as Result<T, Err>
    }

    match<R>(this: Result<Ok, Err>, ok: (ok: Ok) => R, err: (err: Err) => R) {
        if (this.is_ok()) return ok(this.value)
        else return err(this.error)
    }
}

class OK<Ok, Err> extends ResultMethods<Ok, Err> {
    public readonly value: Ok

    constructor(value: Ok) {
        super()
        this.value = value
    }

    and<O>(res: Result<O, Err>): Result<O, Err> {
        return res;
    }

    and_then<O>(op: (value: Ok) => Result<O, Err>): Result<O, Err> {
        return op(this.value)
    }

    or<F>(res: Result<Ok, F>): OK<Ok, F> {
        return this as unknown as OK<Ok, F>;
    }

    or_else<F>(fallback: (err: Err) => Result<Ok, F>): OK<Ok, F> {
        return this as unknown as OK<Ok, F>;
    }

    map<T>(fn: (value: Ok) => T): OK<T, Err> {
        return Result.ok<T, Err>(fn(this.value))
    }

    map_err<T>(fn: (err: Err) => T): OK<Ok, T> {
        return this as unknown as OK<Ok, T>
    }

    expect(error: string): Ok {
        return this.value
    }

    unwrap(): Ok {
        return this.value;
    }
}

class ERR<Ok, Err> extends ResultMethods<Ok, Err> {
    public readonly error: Err

    constructor(error: Err) {
        super()
        this.error = error
    }

    and<O>(res: Result<O, Err>): ERR<O, Err> {
        return this as unknown as ERR<O, Err>;
    }

    and_then<O>(op: (value: Ok) => Result<O, Err>): Result<O, Err> {
        return this as unknown as ERR<O, Err>;
    }

    or<F>(res: Result<Ok, F>): Result<Ok, F> {
        return res;
    }

    or_else<F>(fallback: (err: Err) => Result<Ok, F>): Result<Ok, F> {
        return fallback(this.error);
    }

    map<T>(fn: (value: Ok) => T): ERR<T, Err> {
        return this as unknown as ERR<T, Err>
    }

    map_err<T>(fn: (err: Err) => T): ERR<Ok, T> {
        return Result.err(fn(this.error))
    }

    expect(error: string): never {
        throw new Error(`${error}: ${this.error}`);
    }

    unwrap(): never {
        throw new Error(`${this.error}`);
    }
}

export type Result<Ok, Err> = OK<Ok, Err> | ERR<Ok, Err>

export const Result = {
    ok<Ok, Err>(value: Ok) {
        return new OK<Ok, Err>(value)
    },
    err<Ok, Err>(error: Err) {
        return new ERR<Ok, Err>(error)
    },
}

