pub struct PPool {
    #[cfg(feature = "sqlite")]
    pub pool: sqlx::SqlitePool,
    #[cfg(feature = "postgres")]
    pub pool: sqlx::PgPool,
    #[cfg(feature = "mysql")]
    pub pool: sqlx::MySqlPool,
}

#[cfg(feature = "postgres")]
impl AsRef<sqlx::PgPool> for PPool {
    fn as_ref(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

#[cfg(feature = "sqlite")]
impl AsRef<sqlx::SqlitePool> for PPool {
    fn as_ref(&self) -> &sqlx::SqlitePool {
        &self.pool
    }
}

#[cfg(feature = "mysql")]
impl AsRef<sqlx::MySqlPool> for PPool {
    fn as_ref(&self) -> &sqlx::MySqlPool {
        &self.pool
    }
}
