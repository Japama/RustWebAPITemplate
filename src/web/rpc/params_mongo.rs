use serde::Deserialize;



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
