# dedicated-server-availability-watcher

A simple CLI/daemon tool polling dedicated server availability which optionally notifies.

# sample output

OVH inventory :

    Known servers:
    1801sk14 N/A N/A
    ...
    19fs03-04-apac ram-64g-ecc-1866 hardraid-12x4000sas
    ...
    20adv02-syd ram-64g-ecc-2666 softraid-2x3840nvme
    ...
    21adv01-mum ram-64g-ecc-3200 softraid-3x1920nvme
    ...
    22rise01 ram-32g-ecc-2666 softraid-2x4000sa
    ...
    22sk030 ram-32g-1333 softraid-2x2000sa
    ...
    22sys041 ram-32g-ecc-2133 softraid-2x2000sa
    ...
    23risestorle01 ram-32g-ecc-2666 softraid-3x960nvme

Checking for some OVH server type :

    $ ... check ovh 22sk010 22sk020 22sk030
    22sk030

    # NOTE: here, only `22sk030` is **not** out of stock

Scaleway inventory :

    Known servers:
    23bc9219-0b3a-459a-9eb6-738ebb64f3ab (EM-T510X-NVME) 1536G 20800G
    bd757ca3-a71b-4158-9ef5-39436b6db2a4 (EM-L101X-SATA) 96G 12000G
    a5065ba4-dde2-45f3-adec-1ebbb27b766b (EM-B112X-SSD) 192G 2000G
    d1e32633-3b8a-488c-b1db-8afcc4cdb216 (EM-T210E-NVME) 2048G 7680G
    67ca9c9c-2f4a-447d-8d6d-7d242382a4a3 (EM-B312X-SSD) 256G 2000G
    ddaf8ba6-b2b2-4279-8af3-51930fb602f8 (EM-B212X-SSD) 256G 2000G
    25dcf38b-c90c-4b18-97a2-6956e9d1e113 (EM-A210R-HDD) 16G 2000G
    6ec5bbb2-8267-41cd-aaf6-d43ea1b59372 (EM-A315X-SSD) 64G 2000G
    c5853302-63e4-40c7-a711-4a91629565c8 (EM-L105X-SATA) 96G 24000G
    a204136d-656b-44b7-9735-88ca2f62cb1f (EM-L110X-SATA) 96G 48000G
    5a505e81-5f6a-42bd-acec-0b290be92697 (EM-A410X-SSD) 64G 2000G
    8c2cf291-a3d3-4dc5-967f-f498a088411d (EM-B111X-SATA) 192G 16000G
    c753f736-fbb4-4689-ae93-623f9d08dce5 (EM-B211X-SATA) 256G 16000G
    4635682e-a2ac-4dcd-8071-3deb051e7398 (EM-B311X-SATA) 256G 24000G

Checking for some Scaleway server type :

    $ ... check scaleway a5065ba4-dde2-45f3-adec-1ebbb27b766b 67ca9c9c-2f4a-447d-8d6d-7d242382a4a3
    a5065ba4-dde2-45f3-adec-1ebbb27b766b

    # NOTE: Here, `67ca9c9c-2f4a-447d-8d6d-7d242382a4a3` is absent because it is out of stock.

See `Usage` for the actual commands.

# Compilation

Build for release :

    cargo build --release

Then you will find the dedicated binary in `target/release/`

# Usage

    $ dedicated-server-availability-watcher
    Check and notify about dedicated servers availability

    Usage: dedicated-server-availability-watcher.exe <COMMAND>

    Commands:
    provider  provider actions
    notifier  notifier actions
    help      Print this message or the help of the given subcommand(s)

    Options:
    -h, --help     Print help information
    -V, --version  Print version information

## Notifiers

Listing available notifiers :

    $ dedicated-server-availability-watcher notifier list
    Available notifiers:
    - ifttt-webhook-json
    - ifttt-webhook-values
    - simple-get
    - simple-post
    - simple-put

Testing that a notifier works :

    $ dedicated-server-availability-watcher notifier test NOTIFIER_NAME
    Notification sent

## Providers

Listing available providers :

    $ dedicated-server-availability-watcher provider list
    Available providers:
    - ovh
    - scaleway

Listing a provider inventory :

    $ dedicated-server-availability-watcher provider inventory PROVIDER_NAME [--all]
    Working...
    Known servers:
    ...
    (list of server info, where first column is the SERVER_ID for the `check` function below)
    (if you provide the `--all` option, servers in red color are unavailable)
    (without the option, unavailable servers are simply not listed)
    ...

Checking a provider for a specific server type, with results to `stdout` :

    $ dedicated-server-availability-watcher provider check PROVIDER_NAME SERVER_ID [SERVER_ID...]
    ...
    (one line per available server id)
    (no output if none available)
    ...

You can be notified of the result instead :

    $ ... check PROVIDER_NAME SERVER_ID [SERVER_ID...] --notifier=NOTIFIER_NAME

# Configuration

Every setting is passed through environment variables, which are described

## simple-post

    SIMPLE_URL="http://example.org/test.php"

A `json` payload is sent to the URL :

    {
        "provider_name": "dummy_provider",
        "available_servers": [
            "foo_server",
            "bar_server",
            "baz_server"
        ]
    }

The above payload is pretty-printed here, but it is sent in a compact form.

## simple-put

Identical as `simple-post`, except a `PUT` method is used.

## simple-get

Uses the same `SIMPLE_URL` environment variable as above.

Adds the following environment variables for query parameters :

    SIMPLE_GET_PARAM_NAME_PROVIDER="provider"
    SIMPLE_GET_PARAM_NAME_SERVERS="servers"

This results in `GET` query for the same test payload (previously shown in `simple-post) :

    GET /test.php?servers=foo_server%2Cbar_server%2Cbaz_server&provider=dummy_provider

Here, the server type id are comma separated (`,` is encoded as `%2C`).

## ifttt-webhook-json

**IMPORTANT**: an [IFTTT](https://ifttt.com/) account is required.

- Please create one beforehand if you do not already have one,
- Visit the [Maker WebHook](https://ifttt.com/maker_webhooks) and click on `Documentation`,
- Take note of your `api key` at the top of the page.

Then visit [IFTTT applets](https://ifttt.com/my_applets) and :

- create an applet, click `if`
- select `webhooks` and `Receive a web request with a JSON payload`,
- enter an `event name` and take note of it, then click `create`,
- click `then that`
- select `notifications` for example and click `Send a notification from the IFTTT app`
- design the message as you want it, adding text ..
- where you want it to appear, click `Add ingredient` and select `JsonPayload`
- click `create action`, then `continue` then `finish`

Install the IFTTT smarthone app and login using your user account

Define the environment variables below :

    IFTTT_WEBHOOK_EVENT=your_event_name
    IFTTT_WEBHOOK_KEY=your_api_key

Test the notifier, you should get a notification.

The provided `JsonPayload` is the same as the one described in `simple-post`.

**INFO**: to delete an applet, visit the [difficult-to-find  page on IFTTT](https://ifttt.com/p/username/applets/private).

## ifttt-webhook-values

Same as above, except that you must :

- choose `Receive a web request` when choosing `webhooks` when clicking `if`
- add `Value1` and `Value2` when clicking `Add ingredient` while selecting `notifications`

## scaleway

**IMPORTANT**: a [Scaleway](https://www.scaleway.com/) account is required.

- Please create one beforehand if you do not already have one,
- Visit the [Console](https://console.scaleway.com/)
- Click on your user profile and select `API Keys` ([link](https://console.scaleway.com/iam/api-keys)),
- Click on `Generate API key` and take note of it.

Define the environment variables below :

    SCALEWAY_SECRET_KEY="your_api_key"
    SCALEWAY_BAREMETAL_ZONES="fr-par-1,fr-par-2,nl-ams-1"

**INFO**: the zones variable is a comma `,` separated list of identifiers found in the [official API documentation](https://developers.scaleway.com/en/products/baremetal/api/)

Test the provider by listing its inventory.

## ovh

No environment variable is required to query this particular API endpoint.

Test the provider by listing its inventory.

**INFO**, you can exclude datacenters from the inventory and the check :

    OVH_EXCLUDE_DATACENTER=rbx,fr,ca,bhs

Where each value in the comma separated list will exclude :

- a specific datacenter (`rbx`, `sbg`, `gra`, `bhs` ...)
- the country of the datacenter (`fr`, `ca`, ...)

**WARNING**: excluding a country does not actually exclude its datacenters, you have to exclude both. That is strange, but that is how their api works. And as i have found no API entrypoint to list datacenters or country, i cannot separate both types to filter them out automatically.

And you can explore the [official API](https://api.ovh.com/console/) and create an account if needed.
