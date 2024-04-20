# dedicated-server-availability-watcher

A simple CLI/daemon tool polling dedicated server availability which optionally notifies.

Featured providers: 

- [Online.net](https://online.net/), now known as "[Scaleway Dedibox](https://www.scaleway.com/en/dedibox/)"
- [OVH](https://www.ovhcloud.com/), now known as "OVH Cloud"
- [Scaleway Elastic Metal](https://www.scaleway.com/en/elastic-metal/)

Featured notifiers :

- [IFTTT WebHooks](https://ifttt.com/maker_webhooks) with json, or values
- And "simple" requests (a custom URL using either GET with query parameters, or POST/PUT with json)

# Sample output for each provider

## OVH

Inventory (i removed a bunch of line here where "..." are shown) :

    Known servers:
    1801sk12 (@fr,gra,rbx,sbg) N/A N/A
    ...
    21adv01-mum (@ynm) ram-32g-ecc-3200 softraid-2x4000sa
    ...
    21adv01 (@bhs,fra,gra,lon,rbx,sbg,waw) ram-32g-ecc-3200 softraid-2x6000sa
    ...
    21game01-apac (@sgp) ram-64g-ecc-2400 softraid-2x500nvme
    ...
    21game01 (@bhs,fra,gra,lon,sbg,waw) ram-64g-ecc-2666 softraid-2x500nvme
    ...
    21hci01 (@bhs,fra,gra,lon,rbx,sbg,waw) ram-192g-ecc-2933 softraid-12x3840sas-1x3200nvme
    ...
    22sk030 (@bhs,ca,fr,gra,rbx,sbg) ram-32g-1333 softraid-2x2000sa
    ...
    22skgame01-apac (@sg,sgp) ram-32g-noecc-2133 hybridsoftraid-2x480ssd-1x4000sa
    ...
    23risestorle01 (@bhs,fra,gra,lon,rbx,sbg,waw) ram-32g-ecc-2666 softraid-3x960nvme

Checking for some server type (without notifier) :

    $ ... check ovh 22sk010 22sk020 22sk030
    22sk030

    # NOTE: here, only `22sk030` is **not** out of stock

## Online

Inventory:

    Known servers:
    24006 (Core-2-M-SATA@DC3) 192GB 3x4TB
    ...
    24262 (Core-3-M-SATA@DC3) 192GB 3x6TB
    ...
    24104 (Core-7-M-I@DC2,DC5,AMS1) 192GB 2x1.92TBSSD_NVME
    ...
    23981 (Pro-2-M-SATA@DC3) 32GB 2x2TB
    ...
    24020 (Pro-3-L-SSD-160@DC3) 96GB 2x160GBSSD
    ...
    21329 (Pro-7-M@DC2,DC5,AMS1) 64GB 2x960GBSSD_NVME
    24241 (Pro-7-S@DC2,DC5,AMS1) 32GB 2x480GBSSD_NVME
    24131 (Start-1-L@DC2,DC3) 16GB 2x1TB
    ...
    23997 (Start-3-S-SSD@DC5) 4GB 1x250GBSSD
    24140 (Store-1-L@DC3) 64GB 12x4TB
    24250 (Store-1-S@DC2,DC3) 32GB 2x4TB
    24244 (Store-2-L@DC3,DC5) 64GB 12x4TB
    ...
    24254 (Store-4-L@DC3) 192GB 5x6TB
    24191 (Store-4-M@DC3) 128GB 4x6TB
    24011 (Store-4-XL@DC3,DC5) 96GB 24x500GBSSD
    24033 (Store-4-XXL@DC3,DC5) 96GB 24x1TBSSD

Checking for some server type (without notifier) :

    $ ... check online 21321 23984 23981 24021
    23981

    # NOTE: only 23981 was **not** is out of stock.

## Scaleway

Inventory :

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

Checking for some server type (without notifier) :

    $ ... check scaleway a5065ba4-dde2-45f3-adec-1ebbb27b766b 67ca9c9c-2f4a-447d-8d6d-7d242382a4a3
    a5065ba4-dde2-45f3-adec-1ebbb27b766b

    # NOTE: Here, `67ca9c9c-2f4a-447d-8d6d-7d242382a4a3` is absent because it is out of stock.

See `Usage` for the actual commands.

# Differential notifications using stored hashes

The tool is designed to notify only if something changed when checking.
As such, two designs are possible: a memory state and a resident daemon,
or a run-once job triggered by cron-like tools, which store their states
on disk. I chose the latter, and a writable *directory* must be provided.

This can be done with the `--storage-dir` (or `-s`) option of the `check`
command. Here are some example use of the storage option :

    ... check AAA BBB CCC
    # this writes states to the **working** directory. This means
    # the directory which was in use when the binary has been invoked,
    # and *not* the directory where the binary is stored.

    ... check -s /var/cache/dsaw AAA BBB CCC
    # will write the state hash files into the /var/cache/dsaw,
    # provided that it actually exists **and** is writable by
    # the user running the program.
    # The program will **not** create the directory by itself,
    # nor set or fix permissions.
    # At the very least, it verifies on startup that the specified
    # directory can be reached and is readable.
    
    ... chech --storage-dir /tmp
    # will store all hash files directly in /tmp, which is crude
    # but works, as the files are small and not many, and the
    # only downside is that you could get spurious notifications
    # on reboot as the /tmp directory is usually cleaned upon boot.

# Compilation

Build for release :

    cargo build --release

Then you will find the dedicated binary in `target/release/`

# Docker

Build the image :

    docker build -it dsaw .

Then on runtime :

    # checks if if container exists    
    docker container ls --all -f name=dsaw -q
    
    # if not, create container
    docker create -it --name dsaw \
    dsaw:latest provider check --notifier NOTIFIER_NAME PROVIDER_NAME SERVER_REF

    # if you are using email-sendmail notifier, add the following options :
    --mount type=bind,src=/etc/msmtprc,dst=/etc/msmtprc,readonly \
    --env EMAIL_TO=target_email@example.org --env EMAIL_FROM=account_email@provider.ext \

    # Then run the program (either manually, or using some sort of cron job) :
    docker start -ai dsaw

The program is not a daemon, it does a single execution then exits, preserving system resources.

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
    - online
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

## email-sendmail

First of all, we use `msmtp` as `sendmail` provider.
See the [documentation](https://marlam.de/msmtp/documentation/) for anything related.
How to send email using `msmtp` it outside of the scope of this document and project.

You will first need to :

- to build a system-wide `msmtprc` yourself
- make sure it works : install `msmtp`, `msmtp-mta`, `mailutils` and test it using `mail`
- and make sure the `PATH` the user runs our program has access to `sendmail`,
  which often is located in `sbin` directories (which are not usually in
  non-root users' `PATH`) and that is because the library used to send emails
  only looks for `sendmail` in the provided `PATH`

Then you have to provide the required information :

- define an ENV `EMAIL_FROM` variable (which may be a dummy value like `a@b.c`
  as `msmtp`may replace it by your account's email)
- define an ENV `EMAIL_TO` to where you want the email notifications to go

You can finally test it using :

    dedicated-server-availability-watcher notifier test email-sendmail

If everything is setup correctly (and your provider does not do stupid antispam stuff)
then you should receive a dummy email from your program.

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
