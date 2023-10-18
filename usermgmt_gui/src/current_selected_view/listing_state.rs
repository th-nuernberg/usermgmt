use usermgmt_lib::ldap::LdapSearchResult;

use crate::io_resource_manager::IoResourceManager;

#[derive(Default, Debug)]
pub struct ListingState {
    pub rw_user_name: Option<String>,
    pub rw_pw: Option<String>,
    pub list_ldap_res: IoResourceManager<LdapSearchResult>,
}
