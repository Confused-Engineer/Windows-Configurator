This is a multi-purpose application meant for Windows configuration, troubleshooting, and maintenance.

Windows Setup Usage Example: 
 - Place the Windows Configurator in a folder on a USB with other executables, Powershell/Batch scripts usaed during setup.
 - Run the "Create Config" to make a default configuration profile, then use Config option "Auto discover" to auto-add the scripts and executables in the config file.
 - Transfer the USB to a new windows instance, and use the "Apps" options to install applications and run the Powershell/batch/executables in the folder through the apps GUI
 - Use the Windows options to configure or open common windows settings from one location as opposed to having to navigate thouroughly through Windows settings.

Windows Troubleshooting Usage Examples:
 - Network Device not reachable:
   - Use Ping to insert what should-be the devices IP Address.
   - This will create a new window to monitor if you device can succesfully reach said network device.
 - Encountering Windows Driver/Crashing issues
   - Run SFC, DISM and monitor to see if any errors are found and/or generated.

Windows Maintenance Usage Example:
 - Under the apps section there is a method for updating all apps (that can be updated through winget, which is most)
