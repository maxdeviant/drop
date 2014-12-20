drop
====

滴

Configuration
-------------

In `config.js`, change `storageRoot` to be the folder you want to store your files in.

Nginx Setup
-----------

Make sure you add the following lines in `nginx.conf`:

```
proxy_set_header    X-Real-IP          $remote_addr;
proxy_set_header    X-Forwarded-For    $proxy_add_x_forwarded_for;
```

This will route the requester's IP address through nginx to the app.
