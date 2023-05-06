import {ComponentFor} from "../../framework/builder";
import {graphComponentBuilder} from "./graph-context";
import {create_item_type, GRAPH_ENTITY_TYPE_LABEL, ItemType, ItemTypeError} from "./usecases/item_types";
import {guard} from "../guards";
import {map} from "../iterator";
import {useState} from "react";
import {Result} from "../result";

// I want to see the types that exist
// i want to be able to add types

const addTypeBuilder = graphComponentBuilder
    .withHook("type_name", () => useState(""))
    .withHook("result", () => useState<Result<ItemType, ItemTypeError> | null>(null))

const addType: ComponentFor<typeof addTypeBuilder> = (
    {
        graph,
        type_name: [type_name, set_type_name],
        result: [result, set_result]
    }
) =>
    <form
        onSubmit={e => {
            e.preventDefault()
            let name = type_name.trim();
            if (name.length > 0) {
                set_result(create_item_type(graph.graph, name))
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
        <button disabled={type_name.trim().length === 0}>
            Add
        </button>
    </form>

const AddType = addTypeBuilder.toReactComponent(addType)

const types: ComponentFor<typeof graphComponentBuilder> = ({graph}) =>
    <>
        <h1>Item Types</h1>
        <ul>
            {[...map(
                graph.graph.get_entity_by_label_as<ItemType>(
                    GRAPH_ENTITY_TYPE_LABEL,
                    guard(n => n instanceof ItemType)
                ),
                (t) => <li key={t._$entity_id.id}>{t._$entity_id.id}</li>
            )]}
        </ul>
        <AddType/>
    </>

export const Types = graphComponentBuilder.toReactComponent(types)
