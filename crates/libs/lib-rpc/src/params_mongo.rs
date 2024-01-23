use modql::filter::ListOptions;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_with::{serde_as, OneOrMany};

#[derive(Deserialize)]
pub struct ParamsForCreateMongo<D> {
    pub data: D,
}

#[derive(Deserialize)]
pub struct ParamsForUpdateMongo<D> {
    pub id: String,
    pub data: D,
}

#[derive(Deserialize)]
pub struct ParamsIdedMongo {
    pub id: String,
}

/// Params structure for any RPC List call.
#[serde_as]
#[derive(Deserialize, Default)]
pub struct ParamsListMongo<F>
where
    F: DeserializeOwned,
{
    #[serde_as(deserialize_as = "Option<OneOrMany<_>>")]
    pub filters: Option<Vec<F>>,
    pub list_options: Option<ListOptions>,
}
