#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use std::fs::File;
    use std::io::{self, BufReader, Read};
    use std::time::{Duration, Instant};
    use surrealdb::engine::remote::ws::{Client, Ws};
    use surrealdb::opt::auth::Root;
    use surrealdb::opt::capabilities::Capabilities;
    use surrealdb::{
        engine::local::{Db, Mem},
        opt::Config,
        Surreal,
    };
    use surrealdb::{Connection, Response};

    static MOTION_DATA_FILE_PATH: &str = "data/motion.dat";
    static LTTB_OBJECT: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/lttb-object.surql"));

    static MOTION_TABLE_NAME: &str = "motion";
    static MOTION_TABLE: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/motion-table.surql"));

    static MOTION_FLEXIBLE_TABLE_NAME: &str = "motion_flexible";

    static MOTION_TABLE_FLEXIBLE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/motion-table-flexible.surql"
    ));

    static FLEXIBLE_QUERY: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/query-flexible.surql"));

    static FLEXIBLE_QUERY_LTTB: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/query-flexible-lttb.surql"
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

    #[derive(Debug, Default, Serialize, Deserialize)]
    struct MotionDataFlexible {
        data: Vec<Motion>,
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

    async fn get_db() -> Surreal<Db> {
        let config = Config::default()
            .strict()
            .capabilities(Capabilities::all())
            .query_timeout(Duration::from_secs(100));

        let db = Surreal::new::<Mem>(config)
            .await
            .expect("Failed to create database connection");

        db.query("DEFINE NAMESPACE test;")
            .await
            .expect("Failed to create namespace");
        db.use_ns("test").await.expect("Failed to use namespace");
        db.query("DEFINE DATABASE test;")
            .await
            .expect("Failed to create database");

        db.use_db("test")
            .await
            .expect("Failed to use namespace and database");
        db
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
        db.query(LTTB_OBJECT).await?.check()?;
        Ok(())
    }

    async fn define_motion_table<C: Connection>(db: &Surreal<C>) -> Result<(), surrealdb::Error> {
        for line in MOTION_TABLE.lines() {
            db.query(line).await?.check()?;
        }
        Ok(())
    }

    async fn define_motion_flexible_table<C: Connection>(
        db: &Surreal<C>,
    ) -> Result<(), surrealdb::Error> {
        for line in MOTION_TABLE_FLEXIBLE.lines() {
            db.query(line).await?.check()?;
        }
        Ok(())
    }

    async fn insert_test_data(
        db: &Surreal<Db>,
        motion_data: MotionData,
    ) -> Result<(), surrealdb::Error> {
        let _: Vec<MotionData> = db.create(MOTION_TABLE_NAME).content(motion_data).await?;
        Ok(())
    }

    async fn insert_test_data_flexible<C: Connection>(
        db: &Surreal<C>,
        motion_data: MotionDataFlexible,
    ) -> Result<(), surrealdb::Error> {
        let _: Option<MotionDataFlexible> = db
            .create((MOTION_FLEXIBLE_TABLE_NAME, "id"))
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

    fn read_motion_data_flexible(file_path: &str) -> io::Result<Vec<Motion>> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let mut output = Vec::new();

        for chunk in buffer.chunks_exact(12) {
            let motion = Motion {
                ax: i16::from_le_bytes([chunk[0], chunk[1]]),
                ay: i16::from_le_bytes([chunk[2], chunk[3]]),
                az: i16::from_le_bytes([chunk[4], chunk[5]]),
                gx: i16::from_le_bytes([chunk[6], chunk[7]]),
                gy: i16::from_le_bytes([chunk[8], chunk[9]]),
                gz: i16::from_le_bytes([chunk[10], chunk[11]]),
            };
            output.push(motion);
        }

        Ok(output)
    }

    async fn load_and_insert_motion_data(
        db: &Surreal<Db>,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let motion_data = read_motion_data(file_path)?;
        insert_test_data(db, motion_data).await?;

        Ok(())
    }

    async fn load_and_insert_motion_data_flexible<C: Connection>(
        db: &Surreal<C>,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let motion_data = read_motion_data_flexible(file_path)?;
        let start = Instant::now();
        let motion_data_flexible = MotionDataFlexible { data: motion_data };
        insert_test_data_flexible(db, motion_data_flexible).await?;
        dbg!(start.elapsed());

        Ok(())
    }

    async fn get_ws_db_with_setup() -> Surreal<Client> {
        let db = get_ws_db().await;
        define_functions(&db)
            .await
            .expect("Failed to define functions");
        define_motion_table(&db)
            .await
            .expect("Failed to define motion table");
        define_motion_flexible_table(&db)
            .await
            .expect("Failed to define motion flexible table");
        // load_and_insert_motion_data(&db, MOTION_DATA_FILE_PATH)
        //     .await
        //     .expect("Failed to load and insert motion data");
        load_and_insert_motion_data_flexible(&db, MOTION_DATA_FILE_PATH)
            .await
            .expect("Failed to load and insert motion data");
        db
    }

    #[tokio::test]
    async fn test_define_functions() {
        let db = get_db().await;
        define_functions(&db)
            .await
            .expect("Failed to define functions");
    }

    #[tokio::test]
    async fn test_define_motion_table() {
        let db = get_db().await;
        define_motion_table(&db)
            .await
            .expect("Failed to define motion table");
    }

    #[tokio::test]
    async fn test_load_and_insert_motion_data() {
        let db = get_db().await;
        define_motion_table(&db)
            .await
            .expect("Failed to define motion table");

        load_and_insert_motion_data(&db, MOTION_DATA_FILE_PATH)
            .await
            .expect("Failed to load and insert motion data");
    }

    #[tokio::test]
    async fn test_load_and_insert_motion_data_flexible() {
        let db = get_db().await;
        define_motion_flexible_table(&db)
            .await
            .expect("Failed to define motion flexible table");

        load_and_insert_motion_data_flexible(&db, MOTION_DATA_FILE_PATH)
            .await
            .expect("Failed to load and insert motion data");
    }

    #[tokio::test]
    async fn test_query_flexible() {
        let db = get_ws_db_with_setup().await;

        // let result: Vec<MotionDataFlexible> = db
        //     .select("motion_flexible")
        //     .await
        //     .expect("Failed to select motion_flexible");
        // let result: Response = db.query(FLEXIBLE_QUERY).await.expect("Failed to query");
        let result: Response = db
            .query(FLEXIBLE_QUERY_LTTB)
            .await
            .expect("Failed to query");
        dbg!(result);
    }
}
