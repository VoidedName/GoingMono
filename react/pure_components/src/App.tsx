import {createBrowserRouter, isRouteErrorResponse, Link, Outlet, RouterProvider, useRouteError} from "react-router-dom";
import SimpleCounter from "./examples/simple-counter";
import {componentBuilder, ComponentFor} from "./framework/builder";

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
        element: <Root/>,
        errorElement: <RouterError />,
        children: [
            {
                path: "simple-counter",
                element: <SimpleCounter/>,
            },
        ]
    },
]);

function Root() {
    return <>
        <div id={"sidebar"}>
            <h1>Menu</h1>
            <nav>
                <ul>
                    <li><Link to={""}>Index</Link></li>
                    <li><Link to={"simple-counter"}>Simple Counter</Link></li>
                </ul>
            </nav>
        </div>
        <div id={"content"}>
            <Outlet/>
        </div>
    </>
}

function App() {
    return <RouterProvider router={router}/>
}

export default App
