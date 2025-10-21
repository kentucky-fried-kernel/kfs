# Configuring `devcontainers` without VSCode
We both decided to leave VSCode, however there is one thing we are missing a lot: `devcontainers`. The config is adapted to be runnable without VSCode, but it requires a little bit of manual work.

## Installing the `devcontainers` CLI
Follow [this guide](https://code.visualstudio.com/docs/devcontainers/devcontainer-cli) and you should be all set. You may have to install `npm` and set some `PATH` variables, I'm sure you will manage.

## User Management
The VSCode extension takes care of user management for you, this is not an out-of-the-box `devcontainers` feature. In order to avoid permission conflicts between the container and the host, the container user must have the same `uid:gid` pair as your user on the host.

For this to work, add the following to your `~/.{zsh,bash}rc`:
```sh
export UID=$(id -u $USER)
export GID=$(id -g $USER)
```

This will create two environment variables which can be accessed by the `devcontainers` CLI when building your environment.

## Building and running the `devcontainer`

You should now be all set! Here are the commands you will need to:

Build the devcontainer:
```sh
devcontainer build --workspace-folder .
```

Start the devcontainer:
```sh
devcontainer run --workspace-folder .
```

Enter the devcontainer:
```sh
devcontainer exec --workspace-folder . zsh
```

## Conclusion
This is a very minimal setup. If you use Zed and want to take it further, have a look at [this article](https://dev.to/ale_annini/lightning-fast-development-with-zed-and-dev-containers-1nbd). If you are on NeoVim, I am sorry.
