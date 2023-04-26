import {componentBuilder, ComponentFor} from "../../framework/builder";
import {useEffect, useState} from "react";

const builder = componentBuilder()
    .withProps<{ name: string }>()
    .withHook("counter", () => useState(0))

export const helloWorld: ComponentFor<typeof builder> = (
    {
        name,
        counter: [counter, setCounter]
    }
) => [
    <div>
        <p>Hello {name}</p>
        <button onClick={() => setCounter(prev => (prev + 1) % 100)}>
            {counter}
        </button>
    </div>,

    [() => {
        const id = setTimeout(() => {
            setCounter(prev => (prev + 1) % 100)
        }, 1000)
        return () => clearTimeout(id)
    }, [counter]]
]

const HelloWorld = builder.toReactComponent(helloWorld);

const useCounterState = (init: number) => {
    return useState(init);
}

const useCounter = () => {
    const [counter, setCounter] = useCounterState(0);

    useEffect(() => {
        const id = setTimeout(() => {
            setCounter(prev => (prev + 1) % 100)
        }, 1000)
        return () => {
            clearTimeout(id)
        }
    }, [counter])

    return [counter, setCounter] as const;
}

const ImpureHelloWorld = (p: { name: string }) => {
    const [counter, setCounter] = useCounter()

    return <div>
        <p>Hello {p.name}</p>
        <button onClick={() => setCounter(prev => (prev + 1) % 100)}>
            {counter}
        </button>
    </div>
}

export default function SimpleCounter() {
    return (
        <div className="App">
            <HelloWorld name={"World"}/>
            <ImpureHelloWorld name={"Impure World"}/>
        </div>
    )
}
