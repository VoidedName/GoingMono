import {destructureResult, PureComponent} from "./component";
import {ReactElement, useEffect, useState} from "react";

type KeyValueToObj<Key extends string, Value> =
    { [key in Key & string]: Value }

type StringKeys<T> =
    (keyof T) & string;

export type ComponentFor<T extends ComponentBuilder<any>, Props extends {} = {}> =
    T extends ComponentBuilder<infer State>
        ? PureComponent<Props & StateToProps<State>>
        : never;

/**
 * A component builder allows us to define the states we want to inject from react into a pure component.
 *
 * It's an immutable builder and parts can be reused without any danger.
 *
 * To get a react component out, simply call toReactComponent on a pure component.
 *
 * There seems to be an issue with chaining directly into "toReactComponent" with type inference,
 * simply call it in two steps. `const builder = componentBuilder().with...` and then `const MyReactComponent = builder.toReactComponent(...)`
 */
export type ComponentBuilder<State extends { [key: string]: any } = {}> =
    {
        withState: <T extends string, I extends any>(name: T, init: () => I) => ComponentBuilder<State & KeyValueToObj<T, I>>
        toReactComponent: <Props extends {} = {}>(component: ComponentFor<ComponentBuilder<State>, Props>) => (props: Props) => ReactElement
    }

function capitalize<T extends string>(x: T): Capitalize<T> {
    return x.charAt(0).toUpperCase() + x.slice(1) as Capitalize<T>;
}

type StateToProps<State extends {}> =
    { [K in StringKeys<State>]: State[K] }
    & { [K in StringKeys<State> as `set${Capitalize<K>}`]: (update: (prev: State[K]) => State[K]) => void }

function setDisplayNameFromComponent<T, State>(reactComponent: (p: any) => ReactElement, component: Function) {
    let name;
    if (component.name) {
        name = component.name
    } else {
        name = "Anonymous"
    }
    (reactComponent as unknown as { displayName: string }).displayName = `withState(${name})`;
}

function withState<State extends { [key: string]: any }, T extends {} = {}>(
    state: { [key in StringKeys<State>]: () => State[key] },
    component: PureComponent<T & StateToProps<State>>
) {
    const keys = Object.keys(state) as StringKeys<State>[];
    keys.sort();

    return function ImpureSandwich(p: T) {
        setDisplayNameFromComponent(ImpureSandwich, component);

        const params = {...p} as T & StateToProps<State>

        for (let k of keys) {
            const [reactState, setReactState] = useState(state[k as StringKeys<State>]);

            params[k] = reactState;

            let setter = `set${capitalize(k)}` as `set${Capitalize<typeof k>}`;
            params[setter] = setReactState as typeof params[typeof setter];
        }

        const [jsx, [updateFunction, deps]] = destructureResult(component(params));

        useEffect(() => {
            if (updateFunction) return updateFunction();
        }, deps)

        return jsx;
    };
}

function _componentBuilder<State extends {}>(state: { [key in StringKeys<State>]: () => State[key] }): ComponentBuilder<State> {
    const _withState = <T extends string, I extends any, >(name: T, init: () => I) => _componentBuilder<State & KeyValueToObj<T, I>>({
        ...state,
        [name]: init
    } as { [key in (keyof State) | T]: () => (State & KeyValueToObj<T, I>)[key] });
    const _toReactComponent = <Props extends {} = {}, >(component: ComponentFor<ComponentBuilder<State>, Props>) => withState(state, component)
    return {
        withState: _withState,
        toReactComponent: _toReactComponent as ComponentBuilder<State>["toReactComponent"],
    }
}

export function componentBuilder(): ComponentBuilder {
    return _componentBuilder({});
}

