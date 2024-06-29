#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use futures::future::join_all;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{self, BufReader, Read, Write};
    use std::sync::Arc;
    use surrealdb::engine::remote::ws::{Client, Ws};
    use surrealdb::opt::auth::Root;
    use surrealdb::Connection;
    use surrealdb::Surreal;

    static MOTION_ID: &str = "id";
    static MOTION_DATA_FILE_PATH: &str = "data/motion.dat";
    static LTTB: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/surql/lttb.surql"));
    static LTTB_OBJECT_WITH_ARRAY: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/surql/lttb-fields.surql"
    ));
    static MOTION_TABLE_NAME: &str = "motion";
    static MOTION_TABLE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/surql/motion-table.surql"
    ));

    #[derive(Debug, Default, Serialize, Deserialize)]
    struct MotionData {
        ax: Vec<i16>,
        ay: Vec<i16>,
        az: Vec<i16>,
        gx: Vec<i16>,
        gy: Vec<i16>,
        gz: Vec<i16>,
    }

    impl MotionData {
        fn add_motion(&mut self, motion: Motion) {
            self.ax.push(motion.ax);
            self.ay.push(motion.ay);
            self.az.push(motion.az);
            self.gx.push(motion.gx);
            self.gy.push(motion.gy);
            self.gz.push(motion.gz);
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Motion {
        ax: i16,
        ay: i16,
        az: i16,
        gx: i16,
        gy: i16,
        gz: i16,
    }

    mod result {
        use std::collections::HashMap;

        use super::*;

        #[derive(Debug, Serialize, Deserialize, Deref)]
        pub struct Standard {
            #[serde(rename = "fn::lttb")]
            pub value: Vec<(f32, f32)>,
        }

        #[derive(Debug, Serialize, Deserialize, Deref)]
        pub struct Columns {
            #[serde(rename = "fn::lttb_columns")]
            pub value: Vec<HashMap<String, (f32, f32)>>,
        }
    }

    async fn get_ws_db() -> Surreal<Client> {
        dotenvy::dotenv().ok();

        let db = Surreal::new::<Ws>(std::env::var("SURREALDB_URL").expect("Failed to get url"))
            .await
            .expect("Failed to create database connection");

        db.signin(Root {
            username: &std::env::var("SURREALDB_USER").expect("Failed to get user"),
            password: &std::env::var("SURREALDB_PASSWORD").expect("Failed to get password"),
        })
        .await
        .expect("Failed to sign in");
        let namespace = std::env::var("SURREALDB_NAMESPACE").expect("Failed to get namespace");
        let database = std::env::var("SURREALDB_DATABASE").expect("Failed to get database");

        db.query(format!("DEFINE NAMESPACE {}", database))
            .await
            .expect("Failed to create database");

        db.use_ns(namespace.clone())
            .await
            .expect("Failed to use namespace");

        db.query(format!("DEFINE DATABASE {}", database))
            .await
            .expect("Failed to create database");
        db.use_db(database.clone())
            .await
            .expect("Failed to use database");

        db.query(format!("REMOVE DATABASE {}", namespace))
            .await
            .expect("Failed to create namespace");

        db.query(format!("DEFINE DATABASE {}", database))
            .await
            .expect("Failed to create database");
        db.use_db(database).await.expect("Failed to use database");

        db
    }

    async fn define_functions<C: Connection>(db: &Surreal<C>) -> Result<(), surrealdb::Error> {
        db.query(LTTB).await?.check()?;
        db.query(LTTB_OBJECT_WITH_ARRAY).await?.check()?;
        Ok(())
    }

    async fn define_motion_table<C: Connection>(db: &Surreal<C>) -> Result<(), surrealdb::Error> {
        for line in MOTION_TABLE.lines() {
            db.query(line).await?.check()?;
        }
        Ok(())
    }

    async fn insert_test_data<C: Connection>(
        db: &Surreal<C>,
        id: &str,
        motion_data: &MotionData,
    ) -> Result<(), surrealdb::Error> {
        let _: Option<MotionData> = db
            .create((MOTION_TABLE_NAME, id))
            .content(motion_data)
            .await?;
        Ok(())
    }

    fn read_motion_data(file_path: &str) -> io::Result<MotionData> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let mut motion_data = MotionData::default();
        for chunk in buffer.chunks_exact(12) {
            let motion = Motion {
                ax: i16::from_le_bytes([chunk[0], chunk[1]]),
                ay: i16::from_le_bytes([chunk[2], chunk[3]]),
                az: i16::from_le_bytes([chunk[4], chunk[5]]),
                gx: i16::from_le_bytes([chunk[6], chunk[7]]),
                gy: i16::from_le_bytes([chunk[8], chunk[9]]),
                gz: i16::from_le_bytes([chunk[10], chunk[11]]),
            };
            motion_data.add_motion(motion);
        }

        Ok(motion_data)
    }

    async fn query_lttb(
        db: &Surreal<Client>,
        record_id: &str,
        column: &str,
        n_out: usize,
    ) -> Result<Vec<(f32, f32)>, surrealdb::Error> {
        let query = format!("SELECT fn::lttb({}, {}) FROM {}", column, n_out, record_id);
        let result: Vec<result::Standard> = db.query(query).await?.take(0).expect("Failed to take");
        Ok(result.first().expect("Failed to get").value.to_owned())
    }

    async fn query_lttb_columns(
        db: &Surreal<Client>,
        record_id: &str,
        columns: &[&str],
        n_out: usize,
    ) -> Result<Vec<HashMap<String, (f32, f32)>>, surrealdb::Error> {
        let query = format!(
            "SELECT fn::lttb_fields({}, [{}], {})",
            record_id,
            columns.join(", "),
            n_out,
        );
        dbg!(&query);
        let result: Vec<result::Columns> = db.query(query).await?.take(0).expect("Failed to take");
        Ok(result.first().expect("Failed to get").value.to_owned())
    }

    async fn save_motion_data(data: &MotionData, file_path: &str) -> io::Result<()> {
        let mut file = File::create(file_path)?;
        for i in 0..data.ax.len() {
            file.write_all(&data.ax[i].to_le_bytes())?;
            file.write_all(&data.ay[i].to_le_bytes())?;
            file.write_all(&data.az[i].to_le_bytes())?;
            file.write_all(&data.gx[i].to_le_bytes())?;
            file.write_all(&data.gy[i].to_le_bytes())?;
            file.write_all(&data.gz[i].to_le_bytes())?;
        }
        Ok(())
    }

    async fn load_and_insert_motion_data<C: Connection>(
        db: &Surreal<C>,
        id: &str,
        file_path: &str,
    ) -> Result<MotionData, Box<dyn std::error::Error>> {
        let motion_data = read_motion_data(file_path)?;
        insert_test_data(db, id, &motion_data).await?;

        Ok(motion_data)
    }

    #[tokio::test]
    #[cfg(not(feature = "benchmark"))]
    async fn test_query() {
        let (motion_data, db) = get_ws_db_with_setup().await;
        let db = Arc::new(db);
        let percentages = [80, 50, 20, 10, 5, 1];
        let columns = ["ax", "ay", "az", "gx", "gy", "gz"];

        let total_length = motion_data.ax.len();

        for &p in percentages.iter() {
            let n_out = (total_length * p / 100).max(2); // Calculate number of points based on percentage

            let tasks: Vec<_> = columns
                .iter()
                .map(|&col| {
                    let db = db.clone();
                    async move {
                        query_lttb(db.as_ref(), "motion", col, n_out)
                            .await
                            .expect("Failed to query")
                    }
                })
                .collect();

            let results: Vec<_> = join_all(tasks).await;

            let mut downsampled_data = MotionData::default();

            for (i, res) in results.iter().enumerate() {
                match i {
                    0 => downsampled_data.ax = res.iter().map(|&(_, value)| value as i16).collect(),
                    1 => downsampled_data.ay = res.iter().map(|&(_, value)| value as i16).collect(),
                    2 => downsampled_data.az = res.iter().map(|&(_, value)| value as i16).collect(),
                    3 => downsampled_data.gx = res.iter().map(|&(_, value)| value as i16).collect(),
                    4 => downsampled_data.gy = res.iter().map(|&(_, value)| value as i16).collect(),
                    5 => downsampled_data.gz = res.iter().map(|&(_, value)| value as i16).collect(),
                    _ => (),
                }
            }

            let file_path = format!("data/motion-{}.dat", p);
            save_motion_data(&downsampled_data, &file_path)
                .await
                .expect("Failed to save to file");
        }
    }

    #[tokio::test]
    async fn test_lttb_on_object() {
        let db = get_ws_db().await;
        define_functions(&db)
            .await
            .expect("Failed to define functions");
        define_motion_table(&db)
            .await
            .expect("Failed to define motion table");

        // Insert test data
        let motion_data = MotionData {
            ax: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            ay: vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
            az: vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19],
            gx: vec![19, 17, 15, 13, 11, 9, 7, 5, 3, 1],
            gy: vec![5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
            gz: vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
        };
        insert_test_data(&db, "test_id", &motion_data)
            .await
            .expect("Failed to insert test data");

        let columns = ["ax", "ay", "az", "gx", "gy", "gz"];
        let n_out = 5;
        let result = query_lttb_columns(&db, "test_id", &columns, n_out)
            .await
            .expect("Failed to query");

        dbg!(&result);
    }

    async fn get_ws_db_with_setup() -> (MotionData, Surreal<Client>) {
        let db = get_ws_db().await;
        define_functions(&db)
            .await
            .expect("Failed to define functions");
        define_motion_table(&db)
            .await
            .expect("Failed to define motion table");
        let motion_data = load_and_insert_motion_data(&db, MOTION_ID, MOTION_DATA_FILE_PATH)
            .await
            .expect("Failed to load and insert motion data");
        (motion_data, db)
    }
}
