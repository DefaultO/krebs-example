# krebs-example

A mod for [Krebs](https://gitea.defaul.to/DefaultO/krebs). It lives at `mods/example` in the core, as a git submodule, and it
builds only inside the core. To get it:

```
git clone --recursive https://gitea.defaul.to/DefaultO/krebs.git
```

It shows every kind of thing a mod can register, in one file: an engine hook, an HTTP route, a socket channel, what the client
shows for a custom object, and game code for players' scripts. Copy it to start a mod of your own. The steps are in
[mods/README.md](https://gitea.defaul.to/DefaultO/krebs/src/branch/main/mods/README.md) in the core.

Try it with `krebs serve --mods example`. `GET /api/example/stats` then answers.

The code is under the MIT licence, see `LICENSE`.
