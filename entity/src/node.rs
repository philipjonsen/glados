//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.7
use std::collections::HashMap;

use anyhow::Result;
use enr::NodeId;
use ethereum_types::U256;
use ethportal_api::utils::bytes::hex_encode;

use sea_orm::{
    entity::prelude::*, ActiveValue::NotSet, FromQueryResult, QueryOrder, QuerySelect, Set,
};
use sea_query::Expr;

use lazy_static::lazy_static;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub node_id: Vec<u8>,
    pub node_id_high: i64,
}

impl Model {
    pub fn node_id_as_hex(&self) -> String {
        hex_encode(&self.node_id)
    }

    pub fn get_node_id(&self) -> NodeId {
        let mut node_id = [0u8; 32];
        node_id.copy_from_slice(&self.node_id);
        NodeId::new(&node_id)
    }

    pub fn get_nickname(&self) -> Option<String> {
        NODE_NICKNAME_MAP.get(&self.node_id_as_hex()).cloned()
    }
}

#[derive(FromQueryResult)]
pub struct ModelWithDistance {
    pub id: i32,
    pub node_id: Vec<u8>,
    pub node_id_high: i64,
    pub distance: i64,
}

impl ModelWithDistance {
    pub fn node_id_as_hex(&self) -> String {
        hex_encode(&self.node_id)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::record::Entity")]
    Record,
    #[sea_orm(has_one = "super::client_info::Entity")]
    ClientInfo,
}

impl Related<super::record::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Record.def()
    }
}

impl Related<super::client_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClientInfo.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum ComputedColumn {
    Distance,
}

impl ColumnTrait for ComputedColumn {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        match self {
            Self::Distance => ColumnType::Integer.def(),
        }
    }
}

pub async fn closest_xor(
    node_id: NodeId,
    conn: &DatabaseConnection,
) -> Result<Vec<ModelWithDistance>> {
    let raw_node_id = U256::from_big_endian(&node_id.raw());
    let node_id_high: i64 = (raw_node_id >> 193).as_u64().try_into().unwrap();

    let distance_expression = Expr::cust_with_values(
        "(\"node\".\"node_id_high\" | $1) - (\"node\".\"node_id_high\" & $2)",
        [node_id_high, node_id_high],
    );

    let nodes = Entity::find()
        .column_as(distance_expression, "distance")
        .order_by_asc(Expr::col(ComputedColumn::Distance))
        .limit(100)
        .into_model::<ModelWithDistance>()
        .all(conn)
        .await?;
    Ok(nodes)
}

pub async fn get_or_create(node_id: NodeId, conn: &DatabaseConnection) -> Result<Model> {
    // First try to lookup an existing entry.
    if let Some(node_id_model) = Entity::find()
        .filter(Column::NodeId.eq(node_id.raw().to_vec()))
        .one(conn)
        .await?
    {
        // If there is an existing record, return it
        return Ok(node_id_model);
    }

    // If no record exists, create one and return it
    let raw_node_id = U256::from_big_endian(&node_id.raw());
    let node_id_high: i64 = (raw_node_id >> 193).as_u64().try_into().unwrap();

    let node_id_model = ActiveModel {
        id: NotSet,
        node_id: Set(node_id.raw().into()),
        node_id_high: Set(node_id_high),
    };

    Ok(node_id_model.insert(conn).await?)
}

lazy_static! {
    pub static ref NODE_NICKNAME_MAP: HashMap<String, String> = {
        let mut nicknames = HashMap::<String, String>::new();
        nicknames.insert(
            "0x5ce3d0e0c0617c0a72b0fa237f8d5913fff0ebd89bc54cd68d95e1669cd9306f".to_string(),
            "trin-ams3-2".to_string(),
        );
        nicknames.insert(
            "0x12268d53c44e6c1aae1ec9bc8ab401c4da776fc7fe6f0ad00f273b4adc7ad872".to_string(),
            "trin-nyc1-2".to_string(),
        );
        nicknames.insert(
            "0x290c187ddc44dfeeb2cdf9d30697a030698dfad2d150d037080c523761f410a5".to_string(),
            "trin-sgp1-2".to_string(),
        );
        nicknames.insert(
            "0x794cbd50977f0b202db565d292d0ca8377cebc3497eaea21822afd967e7d2364".to_string(),
            "trin-nyc1-18".to_string(),
        );
        nicknames.insert(
            "0x4c9e465f50376783c3f91e7c3526cf948b1d82938e0fad3ba07b5abff1e2f4e7".to_string(),
            "trin-ams3-13".to_string(),
        );
        nicknames.insert(
            "0x146905875ac8df6dce7d7b93860adf7b06ea051db7dd52c285aa85049f4d8b05".to_string(),
            "trin-nyc1-6".to_string(),
        );
        nicknames.insert(
            "0xc56aea96614f4b1415a637fbbb070ee80ac25a988b41755a0ff6b38e566343a5".to_string(),
            "trin-nyc1-27".to_string(),
        );
        nicknames.insert(
            "0x0361e3264aa208e555e07292320ff4bf8d14c61a1cdc89841b47cd805d4fc338".to_string(),
            "trin-nyc1-4".to_string(),
        );
        nicknames.insert(
            "0xe95d9c2686eb7d7e2fe6767ba7aa3a8bde7b1c2a8d13644b8babaea26b2c0c09".to_string(),
            "trin-ams3-32".to_string(),
        );
        nicknames.insert(
            "0xaec32459fccc67fe540be30d68169b753fe70144fc5596824a623ba5942ee66b".to_string(),
            "trin-sgp1-24".to_string(),
        );
        nicknames.insert(
            "0xba1647ae9a93d575a3c1d700774dc4d6e71fe2b85f2eb90b286d62b6602ed5cf".to_string(),
            "trin-ams3-26".to_string(),
        );
        nicknames.insert(
            "0x8dac04562791694e89b5e28118512aeb60680a0a622484b9be51bfc9a98881d5".to_string(),
            "trin-sgp1-20".to_string(),
        );
        nicknames.insert(
            "0x9ea4a21e76c31532001768d2881ac816a7bcfc96fdb3a618d2243b16ed47a270".to_string(),
            "trin-sgp1-22".to_string(),
        );
        nicknames.insert(
            "0x490af454caeab270b3a75b7f14f1360e98da78aa9587e6d867964d879fa0f917".to_string(),
            "trin-sgp1-12".to_string(),
        );
        nicknames.insert(
            "0x3b84fc0e4399d3239a625b4cf210e3dcdad0884606b5b62b787c7cae0173a5eb".to_string(),
            "trin-ams3-11".to_string(),
        );
        nicknames.insert(
            "0x623d63b5f621a5daa09b110faa38b7e59847c06af90865476850c6e7a70644e6".to_string(),
            "trin-sgp1-15".to_string(),
        );
        nicknames.insert(
            "0x73d13b449b82209e2ba37da292979643d38dfb961e8c28150d94e696b8a12f1c".to_string(),
            "trin-sgp1-17".to_string(),
        );
        nicknames.insert(
            "0x0632ff9510d10a19587db911d6b5d571fc323e50553c3f5efa2a813a0aadb141".to_string(),
            "trin-sgp1-4".to_string(),
        );
        nicknames.insert(
            "0x00aaad5ba24375811f58cd56d04050a1b5238cea340005e6a207ea3653ee9d61".to_string(),
            "trin-ams3-1".to_string(),
        );
        nicknames.insert(
            "0x2dce06489dcb5d604984e9eaa6ab3296b0e3a8c2da612d150ecf0cd736040b2b".to_string(),
            "trin-nyc1-9".to_string(),
        );
        nicknames.insert(
            "0xcb15ca45a4ec0732775e5b1f2152d02194f262ace714298b96e54097a8415ef6".to_string(),
            "trin-ams3-28".to_string(),
        );
        nicknames.insert(
            "0xd9e519275542f78b61bad173a33a9ca047f4b6f2b07fcfd0314be145579ae280".to_string(),
            "trin-sgp1-29".to_string(),
        );
        nicknames.insert(
            "0x30c1ceb0ccc2448a674b73d573924743ce7622ae112b6fe74666fc01ca0cea4f".to_string(),
            "trin-sgp1-9".to_string(),
        );
        nicknames.insert(
            "0x0b5b4972f32734bb5a2c4e80d4805de26068755e26b50317221fcecd8bd8ad9c".to_string(),
            "trin-nyc1-5".to_string(),
        );
        nicknames.insert(
            "0xfae241338ae8cdd6296e7014877a81848b8e385381bbfcd51b745ef94c09343a".to_string(),
            "trin-ams3-35".to_string(),
        );
        nicknames.insert(
            "0xec1a6d36937e5ae64033f65b805597d0175c0fd3bf236d2dd907e71ce3dc3e3a".to_string(),
            "trin-nyc1-32".to_string(),
        );
        nicknames.insert(
            "0xf2e268333e4ce7286925797da743de0271a43c6a4e0fc61310e397782f022be1".to_string(),
            "trin-nyc1-33".to_string(),
        );
        nicknames.insert(
            "0xb4fbb3a59ecbdeb8396ad398e337b6885beeb1cbb0d00ea9867b33ae85811a0a".to_string(),
            "trin-nyc1-25".to_string(),
        );
        nicknames.insert(
            "0xbffe12db62975395c67d0d2b93495da34e84f6d59984506695d3bb59950ef9ea".to_string(),
            "trin-sgp1-26".to_string(),
        );
        nicknames.insert(
            "0xac9c85b2d013331f397a6eb167e61d9b0c78a610e662e4569e3ee8609a37ee43".to_string(),
            "trin-nyc1-24".to_string(),
        );
        nicknames.insert(
            "0xa63686a8535ffda3f9e5bae54b81e747ed51f185dc0d8777c47905580f587c00".to_string(),
            "trin-sgp1-23".to_string(),
        );
        nicknames.insert(
            "0x8fa77313f6ae5536c9466a7bde1348ff10be4c972015efd190804f3b852bc012".to_string(),
            "trin-ams3-21".to_string(),
        );
        nicknames.insert(
            "0x41549f7182d561154aa35b24bba05dbe2085fb349af4ddaaf806ab5ebd180d9b".to_string(),
            "trin-sgp1-11".to_string(),
        );
        nicknames.insert(
            "0x17cefdd4dc9e3bf8128db908803f00e3e547e3b94a189d78f29c5b79dddaf937".to_string(),
            "trin-sgp1-6".to_string(),
        );
        nicknames.insert(
            "0x1f8592ca7afbdf5a0ae4afb79b7af4f2dc82de50432f94517f7d10ac39b24b92".to_string(),
            "trin-sgp1-7".to_string(),
        );
        nicknames.insert(
            "0x2774d76e8d08abf46b93c2c2a82b6828b63c7108b47a2bde40c4a4fb3329fc3a".to_string(),
            "trin-sgp1-8".to_string(),
        );
        nicknames.insert(
            "0xc20a9b0c7405ffc08da06ef9f2ffc44d38d0f2efcc7e3f7eb520625ca892755e".to_string(),
            "trin-ams3-27".to_string(),
        );
        nicknames.insert(
            "0xd3a47990b3ff8f8bf39928dff9739fe676daa65ad51e84d7b0b3199107f64d2f".to_string(),
            "trin-ams3-29".to_string(),
        );
        nicknames.insert(
            "0xde5166991494ce1a8c11eb5f87104e9085a0844833b18543db3a3fc6ed7fed5d".to_string(),
            "trin-nyc1-30".to_string(),
        );
        nicknames.insert(
            "0xe1ccb68a3417504bdbf3ad0df1c06a911bbc1c29fe84df81ef64f28f0727bc86".to_string(),
            "trin-sgp1-30".to_string(),
        );
        nicknames.insert(
            "0xb1f4814c692b165352fdb27964b284d88a588b9f3d734748d3652f311ce15d2f".to_string(),
            "trin-ams3-25".to_string(),
        );
        nicknames.insert(
            "0xa0cd2d177067f663bd06b351219591ec0bab274f020e0964035a3006d47c298a".to_string(),
            "trin-ams3-23".to_string(),
        );
        nicknames.insert(
            "0xa9c1a8e585e1347175444a1252cc3acce9bb69f4e91ca94958700e9ed11ad45b".to_string(),
            "trin-ams3-24".to_string(),
        );
        nicknames.insert(
            "0x984948788690c4f8f13fdb5b7fbb6998f803f7b0deff2d128d9d1cfba49874cb".to_string(),
            "trin-ams3-22".to_string(),
        );
        nicknames.insert(
            "0x8767d6952ab1bea701cbf6fe70922faf63a6ad2c4128f0369f6fe74a331012aa".to_string(),
            "trin-ams3-20".to_string(),
        );
        nicknames.insert(
            "0x8a9e7cabe2611597080e6b6a6d90719d1484af9f0df9f729d776ec92b9acdaf7".to_string(),
            "trin-nyc1-20".to_string(),
        );
        nicknames.insert(
            "0x7f36f2ab4183c5644e999c305a832ed80d1b00c6d12435a39f37dfc8227c34f0".to_string(),
            "trin-ams3-19".to_string(),
        );
        nicknames.insert(
            "0x556d556773b369686055b1a79b52e0629f3cd4369a0ba505c3eae3c4b97fb0c7".to_string(),
            "trin-nyc1-1".to_string(),
        );
        nicknames.insert(
            "0xf8368f60ee86532cd3990806a356468fc12139791ebf1e6b29d30eedab89f841".to_string(),
            "trin-nyc1-34".to_string(),
        );
        nicknames.insert(
            "0x9223dde3ed71f62f938c7fc6841f4ca34048a8d08b79efea689a5546178fa126".to_string(),
            "trin-nyc1-21".to_string(),
        );
        nicknames.insert(
            "0x7cef9dd4ee6549c7847c0f821480cd724c780ff24335c6d90d2c08931935c7ca".to_string(),
            "trin-sgp1-18".to_string(),
        );
        nicknames.insert(
            "0x68ec0487396f017673e26a719d94cc4464e476da0ae3d2aaecf23ceb83389362".to_string(),
            "trin-nyc1-16".to_string(),
        );
        nicknames.insert(
            "0x00b7c1bd815b980a15a100b570f1d18cc161354e3e443bf7662b8cf8f56334cc".to_string(),
            "trin-ams3-4".to_string(),
        );
        nicknames.insert(
            "0x3514da5e6fae802b62dbd381813a4fdd24d208f78506e10e7d94eaac0045354f".to_string(),
            "trin-nyc1-10".to_string(),
        );
        nicknames.insert(
            "0x199af2ea47c67185d35337fa057226f7bfefc0b99f9a7a1cb50ccdc9ced9d5a4".to_string(),
            "trin-ams3-7".to_string(),
        );
        nicknames.insert(
            "0xbc5aea47708d7f5e01f4a21199e9e68aea75a47fca857d96e4fd0ea035b506a1".to_string(),
            "trin-nyc1-26".to_string(),
        );
        nicknames.insert(
            "0x71ada4d33e410cf5b99e854ffe575f7e662b27ed2d6c64c876029e8d2d64f911".to_string(),
            "trin-nyc1-17".to_string(),
        );
        nicknames.insert(
            "0x5d89843cd35753ba791665ac6ff5fb1bec44ae0a90d4a7a9c8c94f9f26ced0c6".to_string(),
            "trin-ams3-15".to_string(),
        );
        nicknames.insert(
            "0x2571e3711ac1dd67a5eca17537d9f04c0af803d0e84544268c80e3c037c2f71e".to_string(),
            "trin-nyc1-8".to_string(),
        );
        nicknames.insert(
            "0x6e96611931a3c7b2d4e9c81c4f5e8e6162e5680fb3a43f41e8cb37ecf1d1616a".to_string(),
            "trin-ams3-17".to_string(),
        );
        nicknames.insert(
            "0x54dd39b782ab69cf2a403c1a5d47533f86f6b9425dd9e1391acac2ad352f5974".to_string(),
            "trin-ams3-14".to_string(),
        );
        nicknames.insert(
            "0x4f725effe6b5b9652c06343fab80123937d94a23695094545816590551c75d86".to_string(),
            "trin-nyc1-13".to_string(),
        );
        nicknames.insert(
            "0x4632ce072a90b4f9db60e98e664357c73df4beab219c2385ee3e4c31f654ce7d".to_string(),
            "trin-nyc1-12".to_string(),
        );
        nicknames.insert(
            "0x5a1252da910bc2b5a0b3799ee41f99af823cea18eeeb683abab5f5e2a61925f6".to_string(),
            "trin-sgp1-14".to_string(),
        );
        nicknames.insert(
            "0x08653f55f8120591717aae0426c10c25f0e6a7db078c2e1516b4f3059cefff35".to_string(),
            "trin-ams3-5".to_string(),
        );
        nicknames.insert(
            "0x112f9793f8396de367ccf3346cf23949380e054645a74f3a5349fcbf275f134e".to_string(),
            "trin-ams3-6".to_string(),
        );
        nicknames.insert(
            "0x2231a55fefd36b87c6efa5e65e15c4dcc75031fa50d5f9d9090b8798a9446585".to_string(),
            "trin-ams3-8".to_string(),
        );
        nicknames.insert(
            "0xc889870632043710fb9014a7e80bb139455f5f238181479311578c464daf30b7".to_string(),
            "trin-sgp1-27".to_string(),
        );
        nicknames.insert(
            "0xd634b398332876cdfdd7d95bdfa157392cbdaa60cd6885f7f0333b8d99f8eecf".to_string(),
            "trin-nyc1-29".to_string(),
        );
        nicknames.insert(
            "0xdb09a78a967871895f39633489236d055b25fb04caec2f57bdeee3646f7a28a1".to_string(),
            "trin-ams3-30".to_string(),
        );
        nicknames.insert(
            "0x444ebedc9cf2c1caf4f28d5caae67276242fb5e0a6400e0a8491e2ed2a17b218".to_string(),
            "trin-ams3-12".to_string(),
        );
        nicknames.insert(
            "0x6530947af4660f597b5c656217c7c8274e54d632c0b9f52c5c6c0851865d3dc6".to_string(),
            "trin-ams3-16".to_string(),
        );
        nicknames.insert(
            "0x572d6806aeaaf0074ce899de10d29df8349e594f4585e6d4deb3e1064b2a1fa3".to_string(),
            "trin-nyc1-14".to_string(),
        );
        nicknames.insert(
            "0x3eb2def71466ddcf233130f016187e4c3d1884e3e7978edd390d8a25fac86e26".to_string(),
            "trin-nyc1-11".to_string(),
        );
        nicknames.insert(
            "0x1cc9a35d8ee4d814eb58817174e5452a616c10c2a1c7c72af5d0d5ada5c8b611".to_string(),
            "trin-nyc1-7".to_string(),
        );
        nicknames.insert(
            "0xd01ef3920ecbb8fecdda9160d4e7a61c8dfc785c1a128a53ccf6c36a535883bd".to_string(),
            "trin-sgp1-28".to_string(),
        );
        nicknames.insert(
            "0xa3ba2c8e50620bb5f3691d6974ff19af90b775ccd4198c016db38b1e9af2b4e8".to_string(),
            "trin-nyc1-23".to_string(),
        );
        nicknames.insert(
            "0xab4ad0cb9fefd03da68bef18ae96e932eebcdb4bb1b4a763e89c5b2bc4212198".to_string(),
            "trin-sgp1-1".to_string(),
        );
        nicknames.insert(
            "0x95f8b17e1700b74f4ebad4bb5de0564673aeb10b199bd19e65426f7642e9fd3c".to_string(),
            "trin-sgp1-21".to_string(),
        );
        nicknames.insert(
            "0x8452ac5bc378ab275a71e3b891bc770305a2c80dfc4e1f761681c33e4921f9ed".to_string(),
            "trin-sgp1-19".to_string(),
        );
        nicknames.insert(
            "0x6bedd98c677c03e71b43b9860f503940fdf0255a0571a3a52881f0ac045c5d89".to_string(),
            "trin-sgp1-16".to_string(),
        );
        nicknames.insert(
            "0x605654e6946d0b0eeb55fcfde27edf897da863fa3451054ad41c0d12071def21".to_string(),
            "trin-nyc1-15".to_string(),
        );
        nicknames.insert(
            "0x5216adc06e287977518fa2eb305bfc81e101a2b07ecf73573851c0dd077ccfa7".to_string(),
            "trin-sgp1-13".to_string(),
        );
        nicknames.insert(
            "0x769f4d5ca128b3b732aae16581296e7f1d236ec4712f3d5f67d74a3f0e560737".to_string(),
            "trin-ams3-18".to_string(),
        );
        nicknames.insert(
            "0x2aa6d8bf210c6680a8a032257391ade71a53be5e9df5704f27a3bfb7f75a205f".to_string(),
            "trin-ams3-9".to_string(),
        );
        nicknames.insert(
            "0x33d0654d3eb85d874b25166f9de0013a331215a4ebf707f157762f1b7ec1008c".to_string(),
            "trin-ams3-10".to_string(),
        );
        nicknames.insert(
            "0x0edacae07480459447ff8938e619086a11535da1843a0f2a2991311c98e612d3".to_string(),
            "trin-sgp1-5".to_string(),
        );
        nicknames.insert(
            "0xcd9e479273522c08a947c90d2fcbb8d99ecf3babd51095d18d6cfb9eb2270126".to_string(),
            "trin-nyc1-28".to_string(),
        );
        nicknames.insert(
            "0xe41979cbd58a0976b24f945d3319b1723a643be425b42df3bf69f8551d8cf32a".to_string(),
            "trin-ams3-31".to_string(),
        );
        nicknames.insert(
            "0x38237bd265aa76632c88d1d05287084bd02bfb09fd7a35b33decb244a46a99df".to_string(),
            "trin-sgp1-10".to_string(),
        );
        nicknames.insert(
            "0xfd01ad3a29d55e4922eb3f647dc29753a0b58327a0d79661cce891a863e6e7af".to_string(),
            "trin-nyc1-35".to_string(),
        );
        nicknames.insert(
            "0xf5c57402370cf9b77101a3b58da8b4527104b34942f53bb942c84955f30d1da5".to_string(),
            "trin-ams3-34".to_string(),
        );
        nicknames.insert(
            "0xefd72017d61349b8d67c9e9133a72752c6bf62ad423736bbca7fffbbeae2e619".to_string(),
            "trin-ams3-33".to_string(),
        );
        nicknames.insert(
            "0xe7bd23ff5cdd02351791da0087bbbfb60067eb6f75b6492ba9c6885e1b694d0a".to_string(),
            "trin-nyc1-31".to_string(),
        );
        nicknames.insert(
            "0xb7a79e63510cc18ed8b6c9c9cdd14d539c582a5a55c3b1b593198f4851bd71b5".to_string(),
            "trin-sgp1-25".to_string(),
        );
        nicknames.insert(
            "0x8141bb6328eaba0c215e6a1aa24d9e63af9c3fe288567e38dd1e9ebaf53bb066".to_string(),
            "trin-nyc1-19".to_string(),
        );
        nicknames.insert(
            "0x9bd5f31dcb19bac10077cc903b651d1885807907b256bb497e683715bac20a66".to_string(),
            "trin-nyc1-22".to_string(),
        );
        nicknames
    };
}
