import {Result} from "./result";
import {Unit} from "./todos/Unit";
import {add_of_set, delete_of_set, get} from "../map";
import {concat, filter, map, unique_by} from "./iterator";
import { v4 as uuid4 } from "uuid"
import {Guard} from "./guards";

type EID = string
type EdgeId = `${EID}:${EID}:${string}`

export class EntityId {
    readonly id: EID

    private constructor(id: EID) {
        this.id = id
    }

    static from_id(id: EID) {
        return new EntityId(id)
    }

    static random() {
        return new EntityId(uuid4())
    }
}

export interface Entity {
    _$entity_id: EntityId
}

export type Edge = Readonly<{ type: string, from: EntityId, to: EntityId }>;

function edge_id(from: EntityId, to: EntityId, type: string): EdgeId {
    return `${from.id}:${to.id}:${type}`;
}

/**
 * Supported operations
 *
 * get entity by id
 *
 * get entity by label
 *
 * get all relations for id
 *
 * get out relations for id
 *
 * get in relations for id
 *
 * get children of id for relation
 *
 * an edge has a single type and goes from a to b, no multi edges
 */
export class Graph {
    private entities = new Map<EID, Entity>()
    private by_label = new Map<string, Set<EID>>
    private labels_for = new Map<EID, Set<string>>

    private edges = new Map<EdgeId, Edge>()
    private out_edges = new Map<EID, Set<EdgeId>>()
    private in_edges = new Map<EID, Set<EdgeId>>()

    remove_labels(e: EntityId) {
        for (const entity of this.get_entity(e)) {
            for (const labels of get(this.labels_for, entity._$entity_id.id)) {
                labels.forEach(label => delete_of_set(this.by_label, label, entity._$entity_id.id))
            }
            this.labels_for.delete(entity._$entity_id.id)
        }
    }

    set_entity(e: Entity, ...labels: string[]) {
        this.remove_labels(e._$entity_id)

        this.entities.set(e._$entity_id.id, e)
        if (labels.length > 0) {
            for (let label of labels) {
                add_of_set(this.by_label, label, e._$entity_id.id)
            }
            this.labels_for.set(e._$entity_id.id, new Set(labels))
        }
    }

    delete_entity(e: EntityId) {
        this.remove_labels(e)
        this.entities.delete(e.id)

        for (const edges of get(this.out_edges, e.id)) {
            edges.forEach(eid => {
                this._delete_edge(eid)
            })
        }

        for (const edges of get(this.in_edges, e.id)) {
            edges.forEach(eid => {
                this._delete_edge(eid)
            })
        }
    }

    get_entity(id: EntityId): Result<Entity, Unit> {
        return get(this.entities, id.id)
    }

    get_entity_as<T extends Entity = Entity>(id: EntityId, guard: (e: Entity) => e is T): Result<T, Unit> {
        return get(this.entities, id.id)
            .and_then((e) => guard(e) ? Result.ok(e) : Result.err(Unit))
    }

    get_entity_by_label(label: string): Iterable<Entity> {
        return get(this.by_label, label)
            .map(ids => map(ids[Symbol.iterator](), id => this.entities.get(id)!))
            .unwrap_or_else(() => [][Symbol.iterator]() as Generator<Entity>)
    }

    get_entity_by_label_as<T extends Entity = Entity>(label: string, guard: Guard<Entity, T>): Iterable<T> {
        return filter<Entity, T>(this.get_entity_by_label(label), guard)
    }

    get_out_edges_of(id: EntityId): Iterable<Edge> {
        return get(this.out_edges, id.id)
            .map(edges => map(edges, (e) => this.edges.get(e)!))
            .unwrap_or_else(() => [][Symbol.iterator]() as Generator<Edge>)
    }

    get_in_edges_of(id: EntityId): Iterable<Edge> {
        return get(this.in_edges, id.id)
            .map(edges => map(edges, (e) => this.edges.get(e)!))
            .unwrap_or_else(() => [][Symbol.iterator]() as Generator<Edge>)
    }

    get_edges_of(id: EntityId): Iterable<Edge> {
        return unique_by(
            concat(this.get_out_edges_of(id), this.get_in_edges_of(id)),
            (e) => edge_id(e.from, e.to, e.type)
        )
    }

    add_edge(type: string, from: EntityId, to: EntityId) {
        const id = edge_id(from, to, type)
        this.edges.set(id, {type, from, to})
        add_of_set(this.out_edges, from.id, id)
        add_of_set(this.in_edges, to.id, id)
    }

    private _delete_edge(eid: EdgeId) {
        for (const edge of get(this.edges, eid)) {
            this.edges.delete(eid)
            delete_of_set(this.out_edges, edge.from.id, eid)
            delete_of_set(this.in_edges, edge.to.id, eid)
        }
    }

    delete_edge(type: string, from: EntityId, to: EntityId) {
        this._delete_edge(edge_id(from, to, type))
    }
}
