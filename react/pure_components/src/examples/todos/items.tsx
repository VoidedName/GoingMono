import {ComponentFor} from "../../framework/builder";
import {graphComponentBuilder} from "./graph-context";
import {
    create_relationship_type,
    create_item,
    create_item_type, get_type_of_item,
    GRAPH_ENTITY_TYPE_LABEL, GRAPH_ITEM_LABEL,
    Item,
    ItemType,
    ItemTypeError
} from "./usecases/item_types";
import {guard} from "../guards";
import {map} from "../iterator";
import {useState} from "react";
import {Result} from "../result";
import {Link} from "react-router-dom";

// I want to see the types that exist
// i want to be able to add types

const addItemBuilder = graphComponentBuilder
    .withHook("type_name", () => useState(""))
    .withHook("name", () => useState(""))
    .withHook("description", () => useState(""))
    .withHook("result", () => useState<Result<Item, ItemTypeError> | null>(null))

const addItem: ComponentFor<typeof addItemBuilder> = (
    {
        graph,
        type_name: [type_name, set_type_name],
        name: [name, set_name],
        description: [description, set_description],
        result: [result, set_result]
    }
) => {
    let types = [...graph.graph.get_entity_by_label_as<ItemType>(GRAPH_ENTITY_TYPE_LABEL, guard(p => p instanceof ItemType))]
    types.sort((a, b) => a._$entity_id.id.localeCompare(b._$entity_id.id))

    let _name = name.trim();
    let _description = description.trim();
    let valid = _name.length > 0 && _description.length > 0 && type_name.length > 0;

    return [<form
        onSubmit={e => {
            e.preventDefault()
            if (valid) {
                set_result(create_item(graph.graph, type_name, name, description))
                graph.refresh()
            }
        }}
    >
        <span className={"input"}>
            <label>Enter item name: </label>
            <input
                type={"text"}
                onChange={(e) => {
                    set_name(e.target.value)
                    set_result(null)
                }}
            />
            <label>Enter item description: </label>
            <input
                type={"text"}
                onChange={(e) => {
                    set_description(e.target.value)
                    set_result(null)
                }}
            />
            {result && result.is_err() && <p className={"error-text"}>{result.error}</p>}
        </span>

        <label>Type: </label>
        <select onChange={(e) => set_type_name(e.target.value)} value={type_name}>
            {types.map((i) => <option key={i._$entity_id.id} value={i._$entity_id.id}>{i._$entity_id.id}</option>)}
        </select>
        <button disabled={!valid}>
            Add
        </button>
    </form>,
        () => {
            if (type_name === "" && types.length > 0) {
                set_type_name(types[0]._$entity_id.id)
            }
        }
    ]
}

const AddItem = addItemBuilder.toReactComponent(addItem)

const types: ComponentFor<typeof graphComponentBuilder> = ({graph}) =>
    <>
        <h1>Item Types</h1>
        <ul>
            {[...map(
                graph.graph.get_entity_by_label_as<Item>(
                    GRAPH_ITEM_LABEL,
                    guard(n => n instanceof Item)
                ),
                (t) => <li key={t._$entity_id.id}>
                    <Link to={`./${t._$entity_id.id}`}>
                    {t._$entity_id.id} - {t.name} - {t.description} - {get_type_of_item(graph.graph, t._$entity_id).unwrap()._$entity_id.id}
                    </Link>
                </li>
            )]}
        </ul>
        <AddItem/>
    </>

export const Items = graphComponentBuilder.toReactComponent(types)
