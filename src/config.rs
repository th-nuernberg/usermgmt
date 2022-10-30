/// Definition of configuration options
pub mod config {
    use serde::{Serialize, Deserialize};


    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct MgmtConfig {
        pub student_default_qos: String,
        pub staff_default_qos: String,
        pub student_qos: Vec<String>,
        pub staff_qos: Vec<String>,
        pub valid_qos: Vec<String>,
        pub valid_slurm_groups: Vec<String>,
        pub objectclass_common: Vec<String>,
        pub compute_nodes: Vec<String>,
        pub login_shell: String, 
        pub student_gid: i32,
        pub staff_gid: i32,
        pub faculty_gid: i32,
        pub sacctmgr_path: String,
        pub ldap_domain_components: String,
        pub ldap_org_unit: String,
        pub ldap_server: String,
        pub home_host: String,
        pub nfs_host: String,
        pub quota_softlimit: String,
        pub quota_hardlimit: String,
        pub quota_nfs_softlimit: String, 
        pub quota_nfs_hardlimit: String,
        pub quota_home_softlimit: String,
        pub quota_home_hardlimit: String,
        pub nfs_root_dir: String,
        pub compute_node_root_dir: String,
        pub filesystem: String,
        pub home_filesystem: String,
        pub nfs_filesystem: String,
        pub default_ssh_user: String,
        pub include_dir_mgmt: bool,
        pub use_homedir_helper: bool,
    }

    impl Default for MgmtConfig {
        fn default() -> Self {
            MgmtConfig {
                student_default_qos: "basic".to_string(), 
                staff_default_qos: "advanced".to_string(),
                student_qos: vec!["interactive".to_string(), "basic".to_string()],
                staff_qos: vec!["interactive".to_string(), "advanced".to_string()],
                valid_qos: vec!["interactive".to_string(), "basic".to_string(), "advanced".to_string()],
                objectclass_common: vec!["inetOrgPerson".to_string(), "ldapPublicKey".to_string(),
                                        "organizationalPerson".to_string(), "person".to_string(), 
                                        "posixAccount".to_string(), "shadowAccount".to_string(), 
                                        "slurmRole".to_string(), "top".to_string()],
                valid_slurm_groups: vec!["staff".to_string(), "student".to_string()],
                login_shell: "/bin/bash".to_string(),
                student_gid: 1002,
                staff_gid: 1001,
                faculty_gid: 1000,
                sacctmgr_path: "/usr/local/bin/sacctmgr".to_string(),
                ldap_domain_components: "".to_string(),
                ldap_org_unit: "people".to_string(),
                ldap_server: "ldap://localhost:389".to_string(),
                home_host: "localhost".to_string(),
                nfs_host: "localhost".to_string(),
                quota_softlimit: "200G".to_string(),
                quota_hardlimit: "220G".to_string(),
                quota_nfs_softlimit: "200G".to_string(),
                quota_nfs_hardlimit: "220G".to_string(),
                quota_home_softlimit: "2G".to_string(),
                quota_home_hardlimit: "3G".to_string(),
                compute_nodes: vec!["localhost".to_string()],
                nfs_root_dir: "".to_string(),
                compute_node_root_dir: "".to_string(),
                include_dir_mgmt: false,
                use_homedir_helper: true,
                filesystem: "".to_string(),
                home_filesystem: "".to_string(),
                nfs_filesystem: "".to_string(),
                default_ssh_user: "root".to_string(),
            }
        }
    }
}