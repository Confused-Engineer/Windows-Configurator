

#[cfg(test)]
mod tests
{
    use windows::{
        core::*, Win32::Foundation::*, Win32::Security::*, Win32::System::Memory::*,
        Win32::System::Threading::*,
    };

    

    #[test]
    fn winget_list_all()
    {
        let wingetall = std::process::Command::new("winget")
            .args(["search", "--query", ""])
            .stdout(std::process::Stdio::piped())
            .output();
        if wingetall.is_err()
        {
            return;
        }

        let output = wingetall.unwrap();

        let outstring = String::from_utf8(output.stdout).unwrap();
        
        

        let mut output = String::new();
        for line in outstring.lines() {
            let mut blank = true;
            output.extend(
                line.split_whitespace()
                    .inspect(|_| blank = false)
                    .flat_map(|word| [word, " "])
            );

            if !blank {
                // Remove extra trailing ' '
                output.pop();
            } else if !line.is_empty() {
                // For the "   " => " " case
                output.push(' ');
            }
            output.push('\n');
        }
    
        // Remove trailing '\n'
        output.pop();

        println!("{}", output);
        
    }

    #[test]
    fn winget_list_all_2()
    {
        let wingetall = std::process::Command::new("winget")
            .args(["search", "--query", ""])
            .stdout(std::process::Stdio::piped())
            .output();
        if wingetall.is_err()
        {
            return;
        }

        let output = wingetall.unwrap();

        let outstring = String::from_utf8(output.stdout).unwrap();

        let mut stringvec: Vec<&str> = outstring.split("\n").collect();

        if stringvec.len() < 5
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
            
            println!("Version: {}", reverse[1]);
            println!("ID: {}", reverse[2]);

            

            let mut string = String::new();
            for x in 3..reverse.len()
            {
                

                string = format!("{} ", reverse[x]) + &string;
            }
            println!("Name: {}", string);
            //println!("Name Rev: {:#?}", reverse[3]);
        }

        
    
        // Remove trailing '\n'
        

        
        
    }

    #[test]
    fn test_priv() -> Result<()>
    {

        

        unsafe {
            let mut token = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;
    
            let mut bytes_required = 0;
            _ = GetTokenInformation(token, TokenPrivileges, None, 0, &mut bytes_required);
    
            let buffer = Owned::new(LocalAlloc(LPTR, bytes_required as usize)?);
    
            GetTokenInformation(
                token,
                TokenPrivileges,
                Some(buffer.0 as *mut _),
                bytes_required,
                &mut bytes_required,
            )?;
    
            let header = &*(buffer.0 as *const TOKEN_PRIVILEGES);
    
            let privileges =
                std::slice::from_raw_parts(header.Privileges.as_ptr(), header.PrivilegeCount as usize);
    
            for privilege in privileges {
                let mut name_len = 0;
                _ = LookupPrivilegeNameW(None, &privilege.Luid, PWSTR::null(), &mut name_len);
    
                let mut name = vec![0u16; (name_len + 1) as usize];
                let name = PWSTR(name.as_mut_ptr());
                LookupPrivilegeNameW(None, &privilege.Luid, name, &mut name_len)?;
    
                println!("{}", name.display().to_string())
            }
    
            Ok(())
        }
    }

}