import {destructureResult, PureComponent} from "./component";
import {ReactElement, useEffect} from "react";

type KeyValueToObj<Key extends string, Value> =
    { [key in Key & string]: Value }

type StringKeys<T> =
    (keyof T) & string;

export type ComponentFor<T extends ComponentBuilder<any, any>, CProps extends { [key: string]: any } = {}> =
    T extends ComponentBuilder<infer State, infer Props>
        ? PureComponent<Props & CProps & State>
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
export type ComponentBuilder<State extends { [key: string]: any } = {}, Props extends { [key: string]: any } = {}> =
    {
        withProps: <P extends { [key: string]: any }>() => ComponentBuilder<State, Props & P>

        withHook: <
            Name extends string,
            HReturn extends any
        >(
            name: Name,
            hook: (props: Props & State) => HReturn
        ) => ComponentBuilder<State & KeyValueToObj<Name, HReturn>, Props>

        toReactComponent: <CProps extends { [key: string]: any } = {}>(
            component: ComponentFor<ComponentBuilder<State, Props>, CProps>
        ) => (props: Props & CProps) => ReactElement
    }

function setDisplayNameFromComponent<T, State>(reactComponent: (p: any) => ReactElement, component: Function) {
    let name;
    if (component.name) {
        name = component.name
    } else {
        name = "Anonymous"
    }
    (reactComponent as unknown as { displayName: string }).displayName = `withHooks(${name})`;
}

function withHooks<
    Hooks extends { [key: string]: any },
    T extends { [key: string]: any } = {}
>(
    hooks: [keyof Hooks & string, (props: T & Hooks) => any][],
    component: PureComponent<T & Hooks>
) {
    return function ImpureSandwich(p: T) {
        setDisplayNameFromComponent(ImpureSandwich, component);

        const params = {...p} as T & Hooks

        for (let [name, hook] of hooks) {
            params[name] = hook(params);
        }

        const [jsx, [updateFunction, deps]] = destructureResult(component(params));

        useEffect(() => {
            if (updateFunction) return updateFunction();
        }, deps)

        return jsx;
    };
}

function _componentBuilder<
    Hooks extends {},
    Props extends {}
>(
    hooks: [StringKeys<Hooks>, (p: Hooks & Props) => Hooks[keyof Hooks]][]
): ComponentBuilder<Hooks, Props> {
    return {
        withProps: <P, >() => _componentBuilder<Hooks, Props & P>(hooks),

        withHook: (name, hook) => _componentBuilder<
            Hooks & KeyValueToObj<typeof name, ReturnType<typeof hook>>,
            Props
        >([...hooks, [name, hook]] as any),

        toReactComponent: (comp) => withHooks<Hooks, Props>(hooks, comp as any),
    }
}

export function componentBuilder(): ComponentBuilder {
    return _componentBuilder([]);
}

