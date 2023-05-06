// noinspection LoopStatementThatDoesntLoopJS

import {Edge, Entity, EntityId, Graph} from "../../Graph";
import {Result} from "../../result";
import {guard, Guard} from "../../guards";
import {filter, map} from "../../iterator";

const ItemTypeError = {
    already_exists: "Node with same name already exists!",
    no_such_type: "No such type exists!",
    no_such_item: "No such item exists!",
    type_is_attached: "This type still has edges attached to it!"
} as const
export type ItemTypeError = typeof ItemTypeError[keyof typeof ItemTypeError]

export class ItemType implements Entity {
    _$entity_id: EntityId;

    constructor(id: EntityId) {
        this._$entity_id = id
    }
}

export class Item implements Entity {
    _$entity_id: EntityId
    name: string
    description: string

    constructor(id: EntityId, name: string, description: string) {
        this._$entity_id = id
        this.name = name;
        this.description = description;
    }
}

export class RelationShipType implements Entity {
    _$entity_id: EntityId

    constructor(id: EntityId) {
        this._$entity_id = id
    }
}

export class RelationShip implements Entity {
    _$entity_id: EntityId

    constructor(id: EntityId) {
        this._$entity_id = id
    }
}

export const GRAPH_ENTITY_TYPE_LABEL = "_$entity_type";
export const GRAPH_RELATIONSHIP_TYPE_LABEL = "_$relationship_type";
export const GRAPH_ITEM_LABEL = "_$item"
export const GRAPH_RELATIONSHIP_LABEL = "_$relationship"
export const GRAPH_IS_TYPE = "is_type";
export const GRAPH_HAS_RELATIONSHIP = "has_relationship";
export const GRAPH_RELATIONSHIP_TARGET = "has_target";

function pick_id(graph: Graph) {
    let id
    do {
        id = EntityId.random()
    } while (graph.get_entity(id).is_ok())
    return id
}

export function create_item_type(graph: Graph, name: string): Result<ItemType, ItemTypeError> {
    const item = new ItemType(EntityId.from_id(name));
    return graph.get_entity(item._$entity_id).match(
        () => Result.err(ItemTypeError.already_exists) as Result<ItemType, ItemTypeError>,
        () => {
            graph.set_entity(item, GRAPH_ENTITY_TYPE_LABEL)
            return Result.ok(item)
        }
    )
}

function get_item_type(graph: Graph, name: string): Result<ItemType, ItemTypeError> {
    return graph.get_entity_as(EntityId.from_id(name), ((e) => e instanceof ItemType) as Guard<Entity, ItemType>)
        .map_err(() => ItemTypeError.no_such_type) as Result<ItemType, ItemTypeError>
}

export function delete_item_type(graph: Graph, name: string): Result<ItemType, ItemTypeError> {
    let id = EntityId.from_id(name)
    let data = graph.get_entity_as(id, ((e) => e instanceof ItemType) as Guard<Entity, ItemType>)

    for (const node of data) {
        let edges = graph.get_edges_of(id)
        for (const e of edges) {
            return Result.err(ItemTypeError.type_is_attached) as Result<ItemType, ItemTypeError>
        }

        graph.delete_entity(id)
        return Result.ok(data.unwrap())
    }

    return Result.err(ItemTypeError.no_such_type) as Result<ItemType, ItemTypeError>
}

export function create_item(graph: Graph, type: string, name: string, description: string): Result<Item, ItemTypeError> {
    const id = pick_id(graph)
    const item = new Item(id, name, description)
    let $type = get_item_type(graph, type)
    return $type.match(
        (t) => {
            graph.set_entity(item, GRAPH_ITEM_LABEL)
            graph.add_edge(GRAPH_IS_TYPE, item._$entity_id, t._$entity_id)
            return Result.ok(item)
        },
        () => $type as unknown as Result<Item, ItemTypeError>
    )
}

export function get_all_items_of_type(graph: Graph, type: string): Iterable<Item> {
    return get_item_type(graph, type)
        .map(t => filter(
            graph.get_in_edges_of(t._$entity_id),
            ((edge) => edge.type === GRAPH_IS_TYPE) as Guard<Edge, Edge>)
        )
        .map(edges => map(
            edges,
            (edge) => graph.get_entity_as(edge.from, ((e) => e instanceof Item) as Guard<Entity, Item>))
        )
        .map(nodes => map(
            filter(
                nodes,
                (n) => n.is_ok()),
            n => n.unwrap())
        )
        .unwrap_or_else(() => [][Symbol.iterator]() as Generator<Item>)
}

export function create_relationship_type(graph: Graph, name: string, domain: string, range: string): Result<RelationShipType, ItemTypeError> {
    if (graph.get_entity(EntityId.from_id(name)).is_ok()) return Result.err(ItemTypeError.already_exists) as Result<RelationShipType, ItemTypeError>
    const $domain = graph.get_entity_as(EntityId.from_id(domain), (e => e instanceof ItemType) as Guard<Entity, ItemType>)
    const $range = graph.get_entity_as(EntityId.from_id(range), (e => e instanceof ItemType) as Guard<Entity, ItemType>)
    if ($domain.is_err() || $range.is_err()) return Result.err(ItemTypeError.no_such_type) as Result<RelationShipType, ItemTypeError>

    const $rel = new RelationShipType(EntityId.from_id(name))

    graph.set_entity($rel, GRAPH_RELATIONSHIP_TYPE_LABEL)
    graph.add_edge("domain", $rel._$entity_id, $domain.unwrap()._$entity_id)
    graph.add_edge("range", $rel._$entity_id, $range.unwrap()._$entity_id)
    return Result.ok($rel)
}

export function create_relationship(graph: Graph, type: string, from: EntityId, to: EntityId): Result<RelationShip, ItemTypeError> {
    const $type = graph.get_entity_as(EntityId.from_id(type), (n => n instanceof RelationShipType) as Guard<Entity, RelationShipType>)
    if ($type.is_err()) return Result.err(ItemTypeError.no_such_type) as Result<RelationShip, ItemTypeError>

    const $from = graph.get_entity_as(from, (n => n instanceof Item) as Guard<Entity, Item>)
    const $to = graph.get_entity_as(from, (n => n instanceof Item) as Guard<Entity, Item>)

    if ($from.is_err() || $to.is_err()) return Result.err(ItemTypeError.no_such_type) as Result<RelationShip, ItemTypeError>

    const $rel = new RelationShip(pick_id(graph))

    graph.set_entity($rel, GRAPH_RELATIONSHIP_LABEL)
    graph.add_edge(GRAPH_IS_TYPE, $type.unwrap()._$entity_id, $rel._$entity_id)
    graph.add_edge(GRAPH_HAS_RELATIONSHIP, from, $rel._$entity_id)
    graph.add_edge(GRAPH_RELATIONSHIP_TARGET, $rel._$entity_id, to)
    return Result.ok($rel)
}

export function get_type_of_item(graph: Graph, id: EntityId): Result<ItemType, ItemTypeError> {
    const type_edges = filter(graph.get_out_edges_of(id), (it) => it.type === GRAPH_IS_TYPE)
    const types = map(type_edges, (e) => graph.get_entity_as<ItemType>(e.to, guard(n => n instanceof ItemType)))
    const type = [...types][0]
    if (!type) return Result.err(ItemTypeError.no_such_type) as Result<ItemType, ItemTypeError>

    return type.map_err(_ => ItemTypeError.no_such_type) as Result<ItemType, ItemTypeError>
}
