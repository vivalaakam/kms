pub use sea_orm_migration::prelude::*;

mod m20240714_063538_users;
mod m20240714_083102_keys;
mod m20240714_102309_shares;
mod m20240714_173831_logs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240714_063538_users::Migration),
            Box::new(m20240714_083102_keys::Migration),
            Box::new(m20240714_102309_shares::Migration),
            Box::new(m20240714_173831_logs::Migration),
        ]
    }
}
