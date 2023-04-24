import {ReactElement} from "react";

type PureComponentUpdateFunction =
    (() => void)
    | (() => () => void)

/**
 * Update function to run in a react use effect, can return a cleanup function.
 *
 * An optional dependency array can be provided to be handed to react useEffect. If omitted, the update will run every time.
 */
export type PureComponentUpdate =
    PureComponentUpdateFunction
    | [PureComponentUpdateFunction, any[]]

/**
 * Given the same inputs a pure component should always return the same output. It should also not cause any side effects.
 *
 * React functional components are not pure if they use any of the react hooks,
 * as their return value now depends on the react context
 * and they also cause side effects in the react context.
 * This also means, by extension, that a pure function is not allowed to call any impure functions.
 *
 * In functional programming, when we want to use a pure component but also have mutation,
 * we usually build a sandwich like so.
 *
 * ```ts
 * let input = impure_read()
 * let result = pure_function(input)
 * impure_write(result)
 * ```
 *
 * As we can see, the pure function never touches anything impure.
 *
 * How can we solve this issue in react? One option would be to wrap a pure component with an impure HoC.
 * This HoC can read the values from the react context and write them back to the context after the component finished rendering.
 *
 * Event handlers are usually impure, but are fine in the markdown, since they never get executed by the component and are not scheduled either.
 * (The markdown is the "result" from above, containing the callbacks).
 */
export type PureComponent<T extends {}> =
    ((params: T) => [ReactElement, PureComponentUpdate])
    | ((params: T) => ReactElement)

/**
 * Parses a PureComponent output and returns a more convenient tuple [jsx, updateFunction | undefined]
 */
export function destructureResult(
    result: ReturnType<PureComponent<any>>
): [ReactElement, [PureComponentUpdateFunction, any[] | undefined] | [undefined, undefined]] {
    let [jsx, update] = toRightOptionalPair(result);
    return [jsx, update ? toRightOptionalPair(update) : [undefined, undefined]]
}

/**
 * takes either the left hand of the pair or [left, right] and returns [left, right | undefined]
 *
 * just used to normalize code paths
 */
function toRightOptionalPair<Left, Right>(pairOrLeft: Left | [Left, Right]): [Left, Right | undefined] {
    if (Array.isArray(pairOrLeft)) {
        return pairOrLeft;
    }
    return [pairOrLeft, undefined]
}
