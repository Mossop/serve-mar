# serve-mar

This takes some of the pain out of testing automatic updates for Mozilla
applications.

You still need to [create an update mar file](https://firefox-source-docs.mozilla.org/toolkit/mozapps/update/docs/SettingUpAnUpdateServer.html#obtaining-an-update-mar)
using the instructions provided and be running a build built to support unsigned updates. But once
you have that you can run an update server with:

```
~$ cargo install serve-mar
~$ serve-mar update.mar
```

This will start a basic webserver on port 8000 that serves updates from `/update.xml`.
