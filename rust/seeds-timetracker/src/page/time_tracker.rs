use std::collections::BTreeMap;
use chrono::{DateTime, Local};
use rusty_ulid::Ulid;
use seed::{prelude::*, *};

type ClientId = Ulid;
type ProjectId = Ulid;
type TimeEntryId = Ulid;

// ------ ------
//     Init
// ------ ------

pub fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    Model {
        changes_status: ChangesStatus::NoChanges,
        errors: vec![],
        clients: RemoteData::NotAsked,
        timer_handle: orders.stream_with_handle(streams::interval(1000, || Msg::OnSecondTick)),
    }
}

// ------ ------
//     Model
// ------ ------

pub struct Model {
    changes_status: ChangesStatus,
    errors: Vec<gloo_net::Error>,

    clients: RemoteData<BTreeMap<ClientId, Client>>,
    timer_handle: StreamHandle
}

enum RemoteData<T> {
    NotAsked,
    Loading,
    Loaded(T),
}

enum ChangesStatus {
    NoChanges,
    Saving { requests_in_flight: usize },
    Saved(DateTime<Local>),
}

struct Client {
    name: String,
    projects: BTreeMap<ProjectId, Project>
}

struct Project {
    name: String,
    time_entries: BTreeMap<TimeEntryId, TimeEntry>
}

struct TimeEntry {
    name: String,
    started: DateTime<Local>,
    stopped: Option<DateTime<Local>>
}

// ------ ------
//    Update
// ------ ------

pub enum Msg {
    ClientsFetched(Result<BTreeMap<ClientId, Client>, gloo_net::Error>),
    ChangesSaved(Option<gloo_net::Error>),
    ClearErrors,

    Start(ClientId, ProjectId),
    Stop(ClientId, ProjectId),

    DeleteTimeEntry(ClientId, ProjectId, TimeEntryId),

    TimeEntryNameChanged(ClientId, ProjectId, TimeEntryId, String),
    SaveTimeEntryName(ClientId, ProjectId, TimeEntryId),

    TimeEntryStartedChanged(ClientId, ProjectId, TimeEntryId, String),
    SaveTimeEntryStarted(ClientId, ProjectId, TimeEntryId),

    TimeEntryDurationChanged(ClientId, ProjectId, TimeEntryId, String),

    TimeEntryStoppedChanged(ClientId, ProjectId, TimeEntryId, String),
    SaveTimeEntryStopped(ClientId, ProjectId, TimeEntryId),

    OnSecondTick,
}

pub fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::ClientsFetched(Ok(clients)) => {},
        Msg::ClientsFetched(Err(fetch_error)) => {},

        Msg::ChangesSaved(None) => {},
        Msg::ChangesSaved(Some(fetch_error)) => {},

        Msg::ClearErrors => {},

        Msg::Start(client_id, project_id) => {},
        Msg::Stop(client_id, project_id) => {},

        Msg::DeleteTimeEntry(client_id, project_id, time_entry_id) => {},

        Msg::TimeEntryNameChanged(client_id, project_id, time_entry_id, name) => {},
        Msg::SaveTimeEntryName(client_id, project_id, time_entry_id) => {},

        Msg::TimeEntryStartedChanged(client_id, project_id, time_entry_id, started) => {},
        Msg::SaveTimeEntryStarted(client_id, project_id, time_entry_id) => {},

        Msg::TimeEntryDurationChanged(client_id, project_id, time_entry_id, duration) => {},

        Msg::TimeEntryStoppedChanged(client_id, project_id, time_entry_id, stopped) => {},
        Msg::SaveTimeEntryStopped(client_id, project_id, time_entry_id) => {},

        Msg::OnSecondTick => {},
    }
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model) -> Node<Msg> {
    div!["TimeTracker view"]
}
