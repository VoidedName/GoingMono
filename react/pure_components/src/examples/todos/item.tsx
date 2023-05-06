import {graphComponentBuilder} from "./graph-context";
import {GRAPH_RELATIONSHIP_TYPE_LABEL, Item, RelationShip} from "./usecases/item_types";
import {ComponentFor} from "../../framework/builder";
import {filter, map} from "../iterator";
import {guard} from "../guards";
import {EntityId} from "../Graph";

let itemBuilder = graphComponentBuilder
    .withProps<{item_id: string}>()

let item: ComponentFor<typeof itemBuilder> = ({graph, item_id}) => {
    console.log(item_id)
    let item = graph.graph.get_entity_as<Item>(EntityId.from_id(item_id), guard(n => n instanceof Item)).unwrap()
    let relationship_edges = filter(graph.graph.get_out_edges_of(item._$entity_id), guard(edge => edge.type === GRAPH_RELATIONSHIP_TYPE_LABEL))
    let relationships = [...map(relationship_edges, e => ({
        type: e.type,
        to: graph.graph.get_entity_as<Item>(e.to, guard(n => n instanceof Item)).unwrap()
    }))]

    return <div className={"item"}>
        <h2>{item.name}[{item._$entity_id.id}]</h2>
        <p>{item.description}</p>
        <div>
            <h3>Relationships</h3>
            <ul>
                {relationships.map(({type, to}) => <li>{type}: {to.name}[{to._$entity_id.id}]</li>)}
            </ul>
        </div>
    </div>
}

export const ItemComponent = itemBuilder.toReactComponent(item)
