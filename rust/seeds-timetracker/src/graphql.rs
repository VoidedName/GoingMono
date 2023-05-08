use cynic;
use cynic::{GraphQlResponse};
use gloo_net::http::Request;
use serde::de::DeserializeOwned;

pub type Result<T> = std::result::Result<T, GraphQLError>;

pub async fn send_operation<ResponseData: DeserializeOwned>(
    operation: cynic::Operation<ResponseData>
) -> Result<ResponseData>
{
    match Request::post("https://nameless-brook-400398.eu-central-1.aws.cloud.dgraph.io/graphql")
        .json(&operation)?
        .send()
        .await
    {
        Ok(a) => {
            a.json::<GraphQlResponse<ResponseData>>()
                .await
                .map_err(|e| GraphQLError::DecodeError(e))
                .and_then(|x| {
                    if let Some(errors) = x.errors {
                        Err(GraphQLError::ResponseError(errors))
                    } else {
                        Ok(x.data.expect("data, since no errors"))
                    }
                })
        }
        Err(e) => Err(GraphQLError::FetchError(e)),
    }
}

#[derive(Debug)]
pub enum GraphQLError {
    FetchError(gloo_net::Error),
    ResponseError(Vec<cynic::GraphQlError>),
    DecodeError(gloo_net::Error),
}

impl From<gloo_net::Error> for GraphQLError {
    fn from(error: gloo_net::Error) -> Self {
        Self::FetchError(error)
    }
}

mod queries {
    #[cynic::schema_for_derives(
    file = r#"schema.graphql"#,
    module = "schema",
    )]
    pub mod clients_with_projects {
        //    queryClient {
        //         id
        //         name
        //         projects {
        //             id
        //             name
        //         }
        //     }
        use super::schema;

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Query")]
        pub struct QueryClient {
            pub query_client: Option<Vec<Option<Client>>>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Client {
            pub id: String,
            pub name: String,
            pub projects: Vec<Project>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Project {
            pub id: String,
            pub name: String,
        }
    }

    #[cynic::schema_for_derives(
    file = r#"schema.graphql"#,
    module = "schema",
    )]
    mod clients_with_projects_with_time_entries {
        //    queryClient {
        //         id
        //         name
        //         projects {
        //             id
        //             name
        //             time_entries {
        //                 id
        //                 name
        //                 started
        //                 stopped
        //             }
        //         }
        //     }

        use super::schema;

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Query")]
        pub struct QueryClient {
            pub query_client: Option<Vec<Option<Client>>>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Client {
            pub id: String,
            pub name: String,
            pub projects: Vec<Project>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Project {
            pub id: String,
            pub name: String,
            #[cynic(rename = "time_entries")]
            pub time_entries: Vec<TimeEntry>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct TimeEntry {
            pub id: String,
            pub name: String,
            pub started: DateTime,
            pub stopped: Option<DateTime>,
        }

        #[derive(cynic::Scalar, Debug, Clone)]
        pub struct DateTime(pub String);

    }

    #[cynic::schema_for_derives(
    file = r#"schema.graphql"#,
    module = "schema",
    )]
    mod clients_with_time_blocks_and_time_entries {
        //    queryClient {
        //         id
        //         name
        //         time_blocks {
        //             id
        //             name
        //             status
        //             duration
        //             invoice {
        //                 id
        //                 custom_id
        //                 url
        //             }
        //         }
        //         projects {
        //             time_entries {
        //                 started
        //                 stopped
        //             }
        //         }
        //     }
        use super::schema;

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Query")]
        pub struct QueryClient {
            pub query_client: Option<Vec<Option<Client>>>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Client {
            pub id: String,
            pub name: String,
            #[cynic(rename = "time_blocks")]
            pub time_blocks: Vec<TimeBlock>,
            pub projects: Vec<Project>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Project {
            #[cynic(rename = "time_entries")]
            pub time_entries: Vec<TimeEntry>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct TimeEntry {
            pub started: DateTime,
            pub stopped: Option<DateTime>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct TimeBlock {
            pub id: String,
            pub name: String,
            pub status: TimeBlockStatus,
            pub duration: i32,
            pub invoice: Option<Invoice>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Invoice {
            pub id: String,
            #[cynic(rename = "custom_id")]
            pub custom_id: Option<String>,
            pub url: Option<String>,
        }

        #[derive(cynic::Enum, Clone, Copy, Debug)]
        pub enum TimeBlockStatus {
            NonBillable,
            Unpaid,
            Paid,
        }

        #[derive(cynic::Scalar, Debug, Clone)]
        pub struct DateTime(pub String);

    }

    #[allow(non_snake_case, non_camel_case_types)]
    mod schema {
        cynic::use_schema!(r#"schema.graphql"#);
    }
}

