import {createContext, useContext, useState} from "react";
import {Graph} from "../Graph";
import {componentBuilder, ComponentFor} from "../../framework/builder";
import {create_relationship_type, create_item, create_item_type, create_relationship} from "./usecases/item_types";

const GraphContext_ = createContext<{ graph: Graph, refresh: () => void }>(null!)

let contextBuilder = componentBuilder()
    .withProps<{ children: any }>()
    .withHook("graph", () => {
        const [graph, s] = useState({graph: new Graph(), refresh: () => s({...graph})})

        create_item_type(graph.graph, "Task")
        create_item_type(graph.graph, "Story")
        create_relationship_type(graph.graph, "Child", "Story", "Task")
        let task = create_item(graph.graph, "Task", "Task1", "Descr1").unwrap()
        let stroy = create_item(graph.graph, "Story", "Story1", "Descr2").unwrap()
        console.dir(
            create_relationship(graph.graph, "Child", stroy._$entity_id, task._$entity_id).unwrap()
        )
        return graph
    })

let graphContext: ComponentFor<typeof contextBuilder> = ({graph, children}) =>
    <GraphContext_.Provider value={graph}>
        {children}
    </GraphContext_.Provider>;

export const GraphContext = contextBuilder.toReactComponent(graphContext)

export const graphComponentBuilder = componentBuilder()
    .withHook("graph", () => useContext(GraphContext_))
