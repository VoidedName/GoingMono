import {ComponentFor} from "../../framework/builder";
import {graphComponentBuilder} from "./graph-context";
import {
    create_relationship_type,
    create_item_type,
    GRAPH_ENTITY_TYPE_LABEL, GRAPH_RELATIONSHIP_TYPE_LABEL,
    ItemType,
    ItemTypeError, RelationShipType
} from "./usecases/item_types";
import {guard} from "../guards";
import {map} from "../iterator";
import {useState} from "react";
import {Result} from "../result";
import {Entity} from "../Graph";

// I want to see the relationshipTypes that exist
// i want to be able to add relationshipTypes

const addTypeBuilder = graphComponentBuilder
    .withHook("type_name", () => useState(""))
    .withHook("domain", () => useState<string | null>(null))
    .withHook("range", () => useState<string | null>(null))
    .withHook("result", () => useState<Result<ItemType, ItemTypeError> | null>(null))

const addType: ComponentFor<typeof addTypeBuilder> = (
    {
        graph,
        type_name: [type_name, set_type_name],
        domain: [domain, set_domain],
        range: [range, set_range],
        result: [result, set_result]
    }
) => {
    let types = [...graph.graph.get_entity_by_label_as<ItemType>(GRAPH_ENTITY_TYPE_LABEL, guard(p => p instanceof ItemType))]
    types.sort((a, b) => a._$entity_id.id.localeCompare(b._$entity_id.id))
    let name = type_name.trim();
    let valid = name.length > 0 && domain && range;

    return [<form
        onSubmit={e => {
            e.preventDefault()
            if (valid) {
                set_result(create_relationship_type(graph.graph, name, domain!, range!))
                graph.refresh()
            }
        }}
    >
        <span className={"input"}>
            <label>Enter new type name: </label>
            <input
                type={"text"}
                onChange={(e) => {
                    set_type_name(e.target.value)
                    set_result(null)
                }}
            />
            {result && result.is_err() && <p className={"error-text"}>{result.error}</p>}
        </span>

        <label>Domain: </label>
        <select onChange={(e) => set_domain(e.target.value)} value={domain ?? undefined}>
            {types.map((i) => <option key={i._$entity_id.id} value={i._$entity_id.id}>{i._$entity_id.id}</option>)}
        </select>

        <label>Range: </label>
        <select  onChange={(e) => set_range(e.target.value)} value={range ?? undefined}>
            {types.map((i) => <option key={i._$entity_id.id} value={i._$entity_id.id}>{i._$entity_id.id}</option>)}
        </select>
        <button disabled={!valid}>
            Add
        </button>
    </form>,
        () => {
            if (domain === null && types.length > 0) {
                set_domain(types[0]._$entity_id.id)
            }
            if (range === null && types.length > 0) {
                set_range(types[0]._$entity_id.id)
            }
        }
    ]
}

const AddType = addTypeBuilder.toReactComponent(addType)

const relationshipTypes: ComponentFor<typeof graphComponentBuilder> = ({graph}) =>
    <>
        <h1>Relationship Types</h1>
        <ul>
            {[...map(
                graph.graph.get_entity_by_label_as<RelationShipType>(
                    GRAPH_RELATIONSHIP_TYPE_LABEL,
                    guard(n => n instanceof RelationShipType)
                ),
                (t) => <li key={t._$entity_id.id}>{t._$entity_id.id}</li>
            )]}
        </ul>
        <AddType/>
    </>

export const RelationshipTypes = graphComponentBuilder.toReactComponent(relationshipTypes)
