use std::os::windows::process::CommandExt;
use crate::Configurator;

impl Configurator
{
    pub fn start_multithread(&mut self)
    {
        // Winget
        let tx = self.multithread_wingetlist.0.clone();
        std::thread::spawn(move || {
            
            let mut wingetlist: crate::app::WingetList = Vec::new();

            let wingetall = std::process::Command::new("winget")
            .args(["search", "--query", ""])
            .creation_flags(0x08000000)
            .stdout(std::process::Stdio::piped())
            .output();
            
            if wingetall.is_err()
            {
                return;
            }

            let wingetall = wingetall.unwrap();

            let winget_string = String::from_utf8(wingetall.stdout);

            if winget_string.is_err()
            {
                return;
            }

            let winget_string = winget_string.unwrap();

            let mut stringvec: Vec<&str> = winget_string.split("\n").collect();

            if stringvec.len() < 20
            {
                return;
            }


            stringvec.remove(0);
            stringvec.remove(0);
            stringvec.pop();
            
            for line in stringvec {
                
                //println!("{}", line);
                let split: Vec<&str> = line.split_ascii_whitespace().collect();
                
                let reverse: Vec<&str> = split.iter().copied().rev().collect();
                
                //println!("Version: {}", reverse[1]);
                //println!("ID: {}", reverse[2]);

                

                let mut string = String::new();
                for x in 3..reverse.len()
                {
                    

                    string = format!("{} ", reverse[x]) + &string;
                }
                //println!("Name: {}", string);

                wingetlist.push(crate::app::Winget
                {
                    name: string,
                    id: reverse[2].to_owned(),
                    version: reverse[1].to_owned(),
                });
            }

            
            let _ = tx.send(wingetlist);
            return;
        });


        // systeminfo
        let tx = self.multithread_systeminfo.0.clone();
        std::thread::spawn(move || {

            loop {
                let output = std::process::Command::new("systeminfo")
                .creation_flags(0x08000000)
                .stdout(std::process::Stdio::piped())
                .output();
    
                if let Ok(output) = output {
                    if let Ok(var) = String::from_utf8(output.stdout)
                    {
                        let _ = tx.send(var);
                    }
                }
    
                std::thread::sleep(std::time::Duration::from_secs(20));                
            }

        });

        
        // ipconfig
        let tx = self.multithread_ipconfig.0.clone();
        std::thread::spawn(move || {

            loop {
                let output = std::process::Command::new("ipconfig")
                .arg("/all")
                .creation_flags(0x08000000)
                .stdout(std::process::Stdio::piped())
                .output();
    
                if let Ok(output) = output {
                    if let Ok(var) = String::from_utf8(output.stdout)
                    {
                        let _ = tx.send(var);
                    }
                }
    
                std::thread::sleep(std::time::Duration::from_secs(20));
            }
        });

        // arp
        let tx = self.multithread_arp.0.clone();
        std::thread::spawn(move || {

            loop {
                let output = std::process::Command::new("arp")
                .arg("-a")
                .creation_flags(0x08000000)
                .stdout(std::process::Stdio::piped())
                .output();
    
                if let Ok(output) = output {
                    if let Ok(var) = String::from_utf8(output.stdout)
                    {
                        let _ = tx.send(var);
                    }
                }
    
                std::thread::sleep(std::time::Duration::from_secs(20));
            }
        });

        // route table
        let tx = self.multithread_routetable.0.clone();
        std::thread::spawn(move || {

            loop {
                let output = std::process::Command::new("route")
                .arg("print")
                .creation_flags(0x08000000)
                .stdout(std::process::Stdio::piped())
                .output();
    
                if let Ok(output) = output {
                    if let Ok(var) = String::from_utf8(output.stdout)
                    {
                        let _ = tx.send(var);
                    }
                }
    
                std::thread::sleep(std::time::Duration::from_secs(20));
            }
        });
        
        // netstat
        let tx = self.multithread_netstat.0.clone();
        std::thread::spawn(move || {

            loop {
                let output = std::process::Command::new("netstat")
                .arg("-n")
                .creation_flags(0x08000000)
                .stdout(std::process::Stdio::piped())
                .output();
    
                if let Ok(output) = output {
                    if let Ok(var) = String::from_utf8(output.stdout)
                    {
                        let _ = tx.send(var);
                    }
                }
    
                std::thread::sleep(std::time::Duration::from_secs(20));
            }
        });

        // config
        let tx = self.multithread_config.0.clone();
        std::thread::spawn(move || {

            loop {
                if let Ok(config) = ini::Ini::load_from_file_noescape("config.ini")
                {
                    let _ = tx.send(config);
                } 
    
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        });
    }
}