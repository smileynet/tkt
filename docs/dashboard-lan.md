# Dashboard on the LAN

From the checkout, bind the server to this computer's Ethernet or Wi-Fi IPv4
address:

```powershell
node tools/serve-dashboard.mjs --host 192.168.0.187 --port 8765
```

Open <http://192.168.0.187:8765/docs/status.html> from a device on the same LAN.
The current address is specific to this machine and can change with DHCP.
Without `--host`, the command listens on loopback (`127.0.0.1`) only.

The server reads the current page on every request and disables caching, so
dashboard updates need no server restart. It serves the page and explicitly
linked local files within this checkout. It provides no directory listings,
repository browsing or write endpoints. Git metadata and scratch/reference
directories are excluded, including through symlinks escaping the checkout.

The command stays running until stopped. A terminal instance can be stopped with
Ctrl+C. The agent-started instance is supervised by its task session; it is not
an installed Windows service and does not automatically start after reboot.
Its current PID/session details are retained locally in
`.scratch/dashboard-lan.json` while it is active. Verify the command line before
stopping a PID, since Windows may reuse process IDs.

Windows must permit this Node executable on the private network. Existing rules
on this machine already do; no firewall settings were changed. A remote device's
Wi-Fi isolation or routing can still affect access.

On plain HTTP, browsers may decline direct clipboard writes. **Copy answers for
chat** then displays and selects the prepared message for manual copying.
