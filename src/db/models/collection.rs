use serde_json::Value as JsonValue;

use uuid::Uuid;

use super::Organization;

#[derive(Debug, Identifiable, Queryable, Insertable, Associations)]
#[table_name = "collections"]
#[belongs_to(Organization, foreign_key = "org_uuid")]
#[primary_key(uuid)]
pub struct Collection {
    pub uuid: String,
    pub org_uuid: String,
    pub name: String,
}

/// Local methods
impl Collection {
    pub fn new(org_uuid: String, name: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),

            org_uuid,
            name,
        }
    }

    pub fn to_json(&self) -> JsonValue {
        json!({
            "Id": self.uuid,
            "OrganizationId": self.org_uuid,
            "Name": self.name,
            "Object": "collection",
        })
    }
}

use diesel;
use diesel::prelude::*;
use db::DbConn;
use db::schema::*;

/// Database methods
impl Collection {
    pub fn save(&mut self, conn: &DbConn) -> bool {
        match diesel::replace_into(collections::table)
            .values(&*self)
            .execute(&**conn) {
            Ok(1) => true, // One row inserted
            _ => false,
        }
    }

    pub fn delete(self, conn: &DbConn) -> bool {
        match diesel::delete(collections::table.filter(
            collections::uuid.eq(self.uuid)))
            .execute(&**conn) {
            Ok(1) => true, // One row deleted
            _ => false,
        }
    }

    pub fn find_by_uuid(uuid: &str, conn: &DbConn) -> Option<Self> {
        collections::table
            .filter(collections::uuid.eq(uuid))
            .first::<Self>(&**conn).ok()
    }

    pub fn find_by_user_uuid(user_uuid: &str, conn: &DbConn) -> Vec<Self> {
        let mut all_access_collections = users_organizations::table
            .filter(users_organizations::user_uuid.eq(user_uuid))
            .filter(users_organizations::access_all.eq(true))
            .inner_join(collections::table.on(collections::org_uuid.eq(users_organizations::org_uuid)))
            .select(collections::all_columns)
            .load::<Self>(&**conn).expect("Error loading collections");

        let mut assigned_collections = users_collections::table.inner_join(collections::table)
            .filter(users_collections::user_uuid.eq(user_uuid))
            .select(collections::all_columns)
            .load::<Self>(&**conn).expect("Error loading collections");

        all_access_collections.append(&mut assigned_collections);
        all_access_collections
    }

    pub fn find_by_organization_and_user_uuid(org_uuid: &str, user_uuid: &str, conn: &DbConn) -> Vec<Self> {
        Self::find_by_user_uuid(user_uuid, conn).into_iter().filter(|c| c.org_uuid == org_uuid).collect()
    }

    pub fn find_by_uuid_and_user(uuid: &str, user_uuid: &str, conn: &DbConn) -> Option<Self> {
        users_collections::table.inner_join(collections::table)
            .filter(users_collections::collection_uuid.eq(uuid))
            .filter(users_collections::user_uuid.eq(user_uuid))
            .select(collections::all_columns)
            .first::<Self>(&**conn).ok()
    }
}

use super::User; 

#[derive(Debug, Identifiable, Queryable, Insertable, Associations)]
#[table_name = "users_collections"]
#[belongs_to(User, foreign_key = "user_uuid")]
#[belongs_to(Collection, foreign_key = "collection_uuid")]
#[primary_key(user_uuid, collection_uuid)]
pub struct CollectionUsers {
    pub user_uuid: String,
    pub collection_uuid: String,
    pub read_only: bool,
}

/// Database methods
impl CollectionUsers {
    pub fn save(user_uuid: &str, collection_uuid: &str, read_only:bool, conn: &DbConn) -> bool {
        match diesel::replace_into(users_collections::table)
            .values((
                users_collections::user_uuid.eq(user_uuid),
                users_collections::collection_uuid.eq(collection_uuid),
                users_collections::read_only.eq(read_only),
            )).execute(&**conn) {
            Ok(1) => true, // One row inserted
            _ => false,
        }
    }

    pub fn delete(user_uuid: &str, collection_uuid: &str, conn: &DbConn) -> bool {
        match diesel::delete(users_collections::table
            .filter(users_collections::user_uuid.eq(user_uuid))
            .filter(users_collections::collection_uuid.eq(collection_uuid)))
            .execute(&**conn) {
            Ok(1) => true, // One row deleted
            _ => false,
        }
    }
}