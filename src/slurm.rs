pub mod slurm {
    use std::process::Command;

    use crate::{Entity, MgmtConfig};

    pub fn add_slurm_user(entity: &Entity, sacctmgr_path: &str) {
        let output = Command::new(sacctmgr_path)
        .arg("add")
        .arg("user")
        .arg(entity.username.clone())
        .arg(format!("Account={}", entity.group.to_string()))
        .arg("--immediate")
        .output()
        .expect("Unable to execute sacctmgr command. Is the path specified in your config correct?");
    
        println!("add_slurm_user stdout: {}", String::from_utf8_lossy(&output.stdout));
    
        if !output.status.success() {
            // println!("Error during sacctmgr execution");
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

        println!("Modifying user qos");
        modify_qos(entity, sacctmgr_path, true);
        modify_qos(entity, sacctmgr_path, false);
    
    }

    pub fn delete_slurm_user(user: &str, sacctmgr_path: &str) {
        // 		cmd = f'sacctmgr delete user {candidate} --immediate'
        let output = Command::new(sacctmgr_path)
        .arg("delete")
        .arg("user")
        .arg(user)
        .arg("--immediate")
        .output()
        .expect("Unable to execute sacctmgr command. Is the path specified in your config correct?");
    
        println!("delete_slurm_user stdout: {}", String::from_utf8_lossy(&output.stdout));
    
        if !output.status.success() {
            // println!("Error during sacctmgr execution");
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }

    }

    pub fn modify_slurm_user(entity: Entity, config: MgmtConfig) {}

    fn modify_qos(entity: &Entity, sacctmgr_path: &str, default_qos: bool) {
        
        let mut qos_str: String = "defaultQos=".to_owned();
        if default_qos {
            qos_str += &entity.default_qos;
        } else {
            let qos_joined = entity.qos.join(",");
            qos_str = format!("qos={}", qos_joined); 
        }

        let output = Command::new(sacctmgr_path)
        .arg("modify")
        .arg("user")
        .arg(entity.username.clone())
        .arg("set")
        .arg(qos_str)
        .arg("--immediate")
        .output()
        .expect("Unable to execute sacctmgr command. Is the path specified in your config correct?");
    
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    
        if !output.status.success() {
            // println!("Error during sacctmgr execution");
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }
    
    }
}


