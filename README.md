# 🐻‍❄️💌 Emails Microservice
> *charted's email service built in Rust that can be connected via gRPC*

**email-service** is a small microservice to help transfer emails towards other people without trying to implement it in different languages. This is used in [charted-server](https://github.com/charted-dev/charted) for member invitations, passwordless authentication, and more.

The service also comes with pre-made templates that you can override easily from the `./templates` directory to suite your needs. Since this is a microservice that anyone can use, the templates can be customized to your liking.

This repository also comes with the templates what we built for charted via the [react-email](https://www.npmjs.com/package/react-email) NPM library, which is available in [./template-builder](./template-builder).

## Installation
### Docker
To use the microservice with Docker, you will need to have the [Docker Engine](https://docker.com) or [Docker Desktop](https://docker.com/products/docker-desktop) installed on your machine. Once you have Docker installed, you can pull the Docker image from Noelware or GitHub's container registry, depends what you want to run:

- If you wish to run ***only stable builds***, you can use [Noelware's Container Registry](https://cr.noelware.cloud).
- If you really want to run the most cutting edge version of this service, you can do so with the Nightly channel. All nightly builds are only available on [GitHub's container registry](https://github.com/orgs/charted-dev/packages) to not clutter Noelware's registry.

The image consists around multiple tags that are suited for your environment. We build the images with the `linux/amd64` and `linux/arm64` architectures.

- `latest`, `nightly` - The latest versions for each channel (`latest` for the **stable** channel, `nightly` for the **nightly** channel)
-  `alpine` - This tag runs this service with the [Alpine](https://hub.docker.com/_/alpine) image instead of [Ubuntu](https://hub.docker.com/_/ubuntu), which is recommended for production environments since it's more compat and smaller.
- `{version}`, `{version}-nightly` - The **{version}** placeholder is for any specific version of this service to run.
- `{version}-alpine` - Similarly to the stock `alpine` image tag, but uses a specific version of this microservice to run.

As this service doesn't hold any persistence, we will not be requiring it and we do not need any external databases or any other service. Now, we can begin pulling the image from the respected registry:

```shell
# Noelware's Container Registry
$ docker pull cr.noelware.cloud/charted/emails

# GitHub's Container Registry
$ docker pull ghcr.io/charted-dev/email-service
```

Now, we can run the container:

```shell
# Noelware's Container Registry
$ docker run -d -p 32121:32121 --name emails cr.noelware.cloud/charted/emails

# GitHub's Container Registry
$ docker run -d -p 32121:32121 --name emails ghcr.io/charted-dev/email-service
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
