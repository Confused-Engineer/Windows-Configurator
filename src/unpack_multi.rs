use crate::Configurator;

impl Configurator
{
    pub fn unpack_multithread(&mut self)
    {
        
        if let Ok(wingetapps) = self.multithread_wingetlist.1.try_recv() 
        {
            self.wingetlist = wingetapps;
        }

        if let Ok(sysinfo) = self.multithread_systeminfo.1.try_recv() 
        {
            self.systeminfo = sysinfo;
        }

        if let Ok(ipconfig) = self.multithread_ipconfig.1.try_recv() 
        {
            self.ipconfig = ipconfig;
        }

        if let Ok(netstat) = self.multithread_netstat.1.try_recv() 
        {
            self.netstat = netstat;
        }

        if let Ok(routetable) = self.multithread_routetable.1.try_recv() 
        {
            self.routetable = routetable;
        }

        if let Ok(arp) = self.multithread_arp.1.try_recv() 
        {
            self.arp = arp;
        }

        if let Ok(config) = self.multithread_config.1.try_recv()
        {
            self.config = config;
        }
    }
}