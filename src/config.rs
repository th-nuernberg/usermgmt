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
        pub student_gid: i32,
        pub staff_gid: i32,
        pub faculty_gid: i32,
        pub sacctmgr_path: String,
        pub ldap_domain_components: String,
        pub ldif_template_path: String,
    }

    impl Default for MgmtConfig {
        fn default() -> Self {
            MgmtConfig {
                student_default_qos: "basic".to_string(), 
                staff_default_qos: "advanced".to_string(),
                student_qos: vec!["interactive".to_string(), "basic".to_string(), "gpubasic".to_string()],
                staff_qos: vec!["interactive".to_string(), "advanced".to_string(), "gpubasic".to_string()],
                valid_qos: vec!["interactive".to_string(), "basic".to_string(), 
                                "advanced".to_string(), "ultimate".to_string(), 
                                "bigmem".to_string(), "gpubasic".to_string(), 
                                "gpuultimate".to_string()],
                valid_slurm_groups: vec!["staff".to_string(), "student".to_string()],
                student_gid: 1002,
                staff_gid: 1001,
                faculty_gid: 1000,
                sacctmgr_path: "/usr/local/bin/sacctmgr".to_string(),
                ldap_domain_components: "dc=informatik,dc=fh-nuernberg,dc=de".to_string(),
                ldif_template_path: "./ldif".to_string(),
            }
        }
    }
}