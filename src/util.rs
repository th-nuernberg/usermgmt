pub mod io_util {
    use std::fs::{File, self};
    use tempfile::TempDir;
    use std::io::{self, BufRead, Write};
    use std::error::Error;
    use std::path::{Path, PathBuf};

    pub fn make_ldif_template() -> Result<String, Box<dyn Error>> {
        #[cfg(debug_assertions)]
        let template_file_basedir: String = "./ldif".to_owned();
    
        #[cfg(not(debug_assertions))]
        let template_file_basedir = "/etc/usermgmt".to_owned();
    
        fs::create_dir_all(&template_file_basedir).unwrap();
        let file_path = template_file_basedir + "/template.ldif";
        
        if Path::new(&file_path).exists() {
            println!("Template already exists at {file_path}. Not creating a new one.");
            return Ok(file_path);
        }
        let mut file = File::create(&file_path)?;

        let mut template_content: Vec<String> = Vec::new();
        template_content.push("objectClass: inetOrgPerson".to_string());
        template_content.push("objectClass: ldapPublicKey".to_string());
        template_content.push("objectClass: organizationalPerson".to_string());
        template_content.push("objectClass: person".to_string());
        template_content.push("objectClass: posixAccount".to_string());
        template_content.push("objectClass: shadowAccount".to_string());
        template_content.push("objectClass: slurmRole".to_string());
        template_content.push("objectClass: top".to_string());
        template_content.push("loginShell: /bin/bash".to_string());

        println!("Creating LDIF template at {file_path}:");
        for s in template_content.iter() {
            writeln!(file, "{}", s)?;
            println!("{}", s);
        }
        Ok(file_path)
    }

    pub fn read_ldif_template(template_path: &String) -> Vec<String> {
        let mut template = Vec::new();

        if let Ok(lines) = read_lines(template_path) {
            // Consumes the iterator, returns an (Optional) String
            for line in lines {
                if let Ok(elem) = line {
                    // println!("{}", elem);
                    template.push(elem);
                }
            }
        }
        template
    }
    
    pub fn write_tmp_ldif(temp_dir: &TempDir, template_vec: Vec<String>, custom_elems: Vec<String>) -> Result<PathBuf, Box<dyn Error>> {
        let file_path = temp_dir.path().join("tmp.ldif");
        let mut file = File::create(&file_path)?;
        println!("Writing temporary ldif file to {:?}", file_path);
        // let mut file = tempfile()?;
        for s in custom_elems.iter() {
            writeln!(file, "{}", s)?;
            println!("{}", s);
        }

        for s in template_vec.iter() {
            writeln!(file, "{}", s)?;
            println!("{}", s);
        }
        println!("write_tmp_ldif finished");
        Ok(file_path)
    }

    pub fn write_to_tmp_file(temp_dir: &TempDir, content: Vec<String>) -> Result<PathBuf, Box<dyn Error>> {
        let file_path = temp_dir.path().join("tmp.file");
        let mut file = File::create(&file_path)?;
        println!("Writing temporary file to {:?}", file_path);
        for s in content.iter() {
            writeln!(file, "{}", s)?;
            println!("{}", s);
        }
        Ok(file_path)
    }

    // The output is wrapped in a Result to allow matching on errors
    // Returns an Iterator to the Reader of the lines of the file.
    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
        where P: AsRef<Path>, {
            let file = File::open(filename)?;
            Ok(io::BufReader::new(file).lines())
    }

    pub fn user_input() -> String {
        let mut input = String::new();
    
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
    
        input = input.trim().to_string();
        input
    }
}

