/**
 * Goals:
 * - create routerItemWrapperBuilder todo item
 * - items can be related to other items
 * - items have routerItemWrapperBuilder type(s?)
 * - an item type has freely defined properties
 * - freely define relationships between types
 * - build views to display items based on some criteria
 */
import {RouteObject, useParams} from "react-router-dom";
import Navigation from "../../routing";
import {Types} from "./types";
import {GraphContext} from "./graph-context";
import {RelationshipTypes} from "./relationship-types";
import {Items} from "./items";
import {ItemComponent} from "./item";
import {componentBuilder, ComponentFor} from "../../framework/builder";


let routerItemWrapperBuilder = componentBuilder()
    .withHook("item_id", () => useParams().item_id!);

let routerItemWrapper: ComponentFor<typeof routerItemWrapperBuilder> = (p) => <ItemComponent {...p} />

let RouterItemWrapper = routerItemWrapperBuilder.toReactComponent(routerItemWrapper)

const router: RouteObject = {
    path: "todos",
    element:
        <GraphContext>
            <Navigation title={"Todos"} links={[
                {path: "", display: "Index"},
                {path: "types", display: "Types"},
                {path: "relationship-types", display: "Relationship Types"},
                {path: "items", display: "Items"},
            ]}/>
        </GraphContext>,
    children: [
        {
            path: "types",
            element: <Types/>
        },
        {
            path: "relationship-types",
            element: <RelationshipTypes/>
        },
        {
            path: "items",
            element: <Items/>
        },
        {
            path: "items/:item_id",
            element: <RouterItemWrapper />
        }
    ]
}

export const TodoRouter = router;

