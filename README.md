# 🐻‍❄️💌 Emails Microservice
> *charted's email service built in Rust that can be connected via gRPC*

**email-service** is a small microservice to help transfer emails towards other people without trying to implement it in different languages. This is used in [charted-server](https://github.com/charted-dev/charted) for member invitations, passwordless authentication, and more.

## Templates
Starting in v0.2.0, you can now use Git to host your templates and the service will pull them into the filesystem and keep track of them once you start sending users emails!

> Note
> You can still host your templates on the filesystem, just use the `templates.fs` object instead.

To use Git, you must need to have it installed on your system (as the service will require [`libgit2`](https://libgit2.github.com) to pull them), and you can set the `templates.git.repository` to `git://[server]/[owner]/[repo]`:

```yaml
templates:
    git:
        repository: git://github.com/charted-dev/email-templates
        directory: ./dist
        branch: main
```

The server will pull the repository in `/var/lib/noelware/charted/emails/templates` (if on Docker if `templates.directory` is not on the disk), or in the `templates.directory` directory.

### SSH
To use the SSH protocol for Git, you will need to have the keys available on the filesystem. You can use the `templates.git.ssh` object to do so:

```yaml
templates:
    git:
        repository: git://github.com/charted-dev/email-templates
        directory: ./dist
        branch: main
        ssh:
            username: noel # some other username...
            keys:
                - ~/.ssh/id_rsa
```

## Installation
### Docker
To use the microservice with Docker, you will need to have the [Docker Engine](https://docker.com) or [Docker Desktop](https://docker.com/products/docker-desktop) installed on your machine. Once you have Docker installed, you can pull the Docker image from Noelware's container registry.

The image consists around multiple tags that are suited for your environment. We build the images with the `linux/amd64` and `linux/arm64` architectures.

- `latest`, `nightly` - The latest versions for each channel (`latest` for the **stable** channel, `nightly` for the **nightly** channel)
-  `alpine` - This tag runs this service with the [Alpine](https://hub.docker.com/_/alpine) image instead of [Debian](https://hub.docker.com/_/debian), which is recommended for production environments since it's more compat and smaller.
- `{version}`, `{version}-nightly` - The **{version}** placeholder is for any specific version of this service to run.
- `{version}-alpine` - Similarly to the stock `alpine` image tag, but uses a specific version of this microservice to run.

As this service doesn't hold any persistence, we will not be requiring it and we do not need any external databases or any other service. Now, we can begin pulling the image from the respected registry:

```shell
$ docker pull cr.noelware.cloud/charted/emails
```

Now, we can run the container:

```shell
$ docker run -d -p 32121:32121 --name emails cr.noelware.cloud/charted/emails
```

### Docker Compose
This repository also comes with a pre-ready `docker-compose.yml` deployment that can be easily fetched:

```shell
# Linux/macOS - cURL
$ curl -Lo docker-compose.yml https://raw.githubusercontent.com/charted-dev/email-service/main/docker-compose.yml

# Windows with PowerShell
$ irm https://raw.githubusercontent.com/charted-dev/email-service/main/docker-compose.yml | Set-Content -Path ./docker-compose.yml
```

## License
**email-service** is released under the **Apache 2.0** License with love by [Noelware, LLC.](https://noelware.org)! 🐻‍❄️💜
