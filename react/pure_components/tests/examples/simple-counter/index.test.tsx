import {destructureResult} from "../../../src/framework/component";
import {expect, test, vi} from "vitest";
import {fireEvent, render, screen} from "@testing-library/react";
import {helloWorld} from "../../../src/examples/simple-counter";

describe("hello world tests", () => {

    beforeEach(() => {
        vi.useFakeTimers()
    })

    afterEach(() => {
        vi.useRealTimers()
    })

    test("renders text", () => {
        let value = 42;
        let [jsx] = destructureResult(
            helloWorld({
                counter: [value, (f) => {
                    if (typeof f === "function") value = f(value)
                    else value = f
                }], name: "World"
            })
        );

        render(jsx)
        expect(screen.getByText(`42`)).toBeDefined()
        expect(screen.getByText(`Hello World`)).toBeDefined()
    })

    test("click increments state", () => {
        let value = 42;
        let [jsx] = destructureResult(
            helloWorld({
                counter: [value, (f) => {
                    if (typeof f === "function") value = f(value)
                    else value = f
                }], name: "World"
            })
        );

        render(jsx)

        fireEvent.click(screen.getByRole("button"))
        expect(value).toBe(43)
    })

    test("click wraps around on 100", () => {
        let value = 99;
        let [jsx] = destructureResult(
            helloWorld({
                counter: [value, (f) => {
                    if (typeof f === "function") value = f(value)
                    else value = f
                }], name: "World"
            })
        );

        render(jsx)

        fireEvent.click(screen.getByRole("button"))
        expect(value).toBe(0)
    })

    test("auto increments in 1000ms steps", async () => {
        let value = 42;
        let [_, [update, deps]] = destructureResult(
            helloWorld({
                counter: [value, (f) => {
                    if (typeof f === "function") value = f(value)
                    else value = f
                }], name: "World"
            })
        );

        expect(update).toBeDefined()
        expect(deps).toHaveLength(1)
        expect(deps![0]).toBe(42)
        update!()

        vi.advanceTimersByTime(999)
        expect(value).toBe(42)

        vi.advanceTimersByTime(2)
        expect(value).toBe(43)
    })

    test("auto increments wraps around on 100", async () => {
        let value = 99;
        let [_, [update, deps]] = destructureResult(
            helloWorld({
                counter: [value, (f) => {
                    if (typeof f === "function") value = f(value)
                    else value = f
                }], name: "World"
            })
        );

        expect(update).toBeDefined()
        expect(deps).toHaveLength(1)
        expect(deps![0]).toBe(99)
        update!()

        vi.advanceTimersByTime(1001)
        expect(value).toBe(0)
    })
})
