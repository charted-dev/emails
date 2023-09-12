# Helm Chart for [charted-emails](https://github.com/charted-dev/emails)
This is the canonical source for the Helm chart distribution for [charted-emails](https://github.com/charted-dev/emails), by [Noelware, LLC.](https://noelware.org).

## Installation
```sh
$ helm repo add charted https://charts.noelware.org/~/charted
$ helm install charted-emails charted/emails --set global.smtp.host=localhost --set global.smtp.port=25
```

## Parameters
<!-- @noelware/helm/values :: START -->

<!-- @noelware/helm/values :: END -->
