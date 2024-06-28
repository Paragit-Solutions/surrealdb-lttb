#[cfg(test)]
mod tests {
    use surrealdb::{
        engine::local::{Db, Mem},
        opt::Config,
        Surreal,
    };

    static LTTB_OBJECT: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/lttb-object.surql"));

    pub async fn get_db() -> Surreal<Db> {
        let config = Config::default().strict();

        Surreal::new::<Mem>(config)
            .await
            .expect("Failed to create database connection")
    }

    pub async fn define_functions(db: &Surreal<Db>) -> Result<(), surrealdb::Error> {
        Ok(())
    }

    #[tokio::test]
    async fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
