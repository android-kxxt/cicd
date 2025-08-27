# CI/CD

## Changelog

We keep a separate `state` repo that tracks tree snapshots and changelogs.

Whenever a build is started, it should check that whether a previous build of the same device
and release type exists by checking the contents of `state/<device>-<...>/last_build`,
which should be the release tag of that build.

The tree snapshot of a build is stored in `state/<device>-<...>/<build-tag>/snapshot`.
To generate a changelog, the changelog generator is invoked with the old snapshot and the new
snapshot which is obtained on the fly by executing `cicd/snapshot.c`.

Then, the generated changelog should be available in Markdown and HTML format, which should be
stored in `state/<device>-<...>/<build-tag>/NEWS.md` and `state/<device>-<...>/<build-tag>/NEWS.html`.
By combining history changelogs, we can create a changelog web page backed by
`state/<device>-<...>/CHANGELOG.md` or `state/<device>-<...>/CHANGELOG.html`.
