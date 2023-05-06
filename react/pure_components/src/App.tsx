import {createBrowserRouter, isRouteErrorResponse, Link, Outlet, RouterProvider, useRouteError} from "react-router-dom";
import SimpleCounter from "./examples/simple-counter";
import {componentBuilder, ComponentFor} from "./framework/builder";
import {TodoRouter} from "./examples/todos";
import Navigation from "./routing";

const routerErrorBuilder = componentBuilder()
    .withHook("error", useRouteError)

const routerError: ComponentFor<typeof routerErrorBuilder> = ({error}) => {
    if (isRouteErrorResponse(error)) {
        return <div id={"error-page"}>
            <h1>{error.status} - {error.statusText}</h1>
            {error.error?.message}
        </div>
    }

    // Last resort fallback
    console.error("Something unexpected happened", error)

    return <div id={"error-page"}>
        <h1>v_v' something bad happened...</h1>
        This is a catch all error page, if you're seeing this, it means this error is unexpected and a bug!
    </div>
}

const RouterError = routerErrorBuilder.toReactComponent(routerError)

const router = createBrowserRouter([
    {
        path: "/",
        element: <Navigation title={"Menu"} links={[
            {path: "", display: "Index"},
            {path: "simple-counter", display: "Simple Counter"},
            {path: "todos", display: "Todos", always_active: true},
        ]} />,
        errorElement: <RouterError />,
        children: [
            {
                path: "simple-counter",
                element: <SimpleCounter/>,
            },
            TodoRouter,
        ]
    },
]);

function App() {
    return <RouterProvider router={router}/>
}

export default App
