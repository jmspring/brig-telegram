---
id: bt-p0rt
status: done
deps: []
links: []
created: 2026-04-14T12:00:00Z
type: feature
priority: 2
assignee: Jim Spring
tags: [packaging, freebsd, ports]
---
# FreeBSD port infrastructure for brig-telegram

## Goal

Add BSD Makefile, port skeleton, and optional rc.d script so that brig-telegram
can be installed following FreeBSD conventions.  The default install path assumes
jailed mode (managed by brig).  Host-mode operation is opt-in via a separate
make target.

## Context

brig-telegram is a ~350-line synchronous Telegram gateway that long-polls
getUpdates and bridges messages to brig's unix domain socket.  It currently has
no Makefile or install target — the README documents a manual `cargo build && cp`
workflow followed by `brig skill add ./` and `brig skill enable`.

brig itself already has a BSD Makefile (`brig/Makefile`) and an rc.d script
(`brig/scripts/rc.d/brig`) that serve as the template for this work.

### Two deployment modes

1. **Jailed (default)** — brig manages the gateway inside a FreeBSD jail.
   The port installs the binary and manifest; the operator runs
   `brig skill enable telegram-gateway` which creates the ZFS dataset,
   jail.conf, pf rules, and its own rc.d script.  The binary at
   `/usr/local/bin/brig-telegram` is nullfs-mounted read-only into the jail.

2. **Host-mode (opt-in)** — the gateway runs directly on the host as a
   standard FreeBSD service.  An rc.d script is installed via
   `make install-service`.  The operator configures rc.conf variables and
   uses `service brig_telegram start`.  No jail, no pf, no ZFS — just the
   binary talking to brig's unix socket.

### Existing manifest.toml

The manifest already exists at `brig-telegram/manifest.toml`:

```toml
[skill]
name = "telegram-gateway"
description = "Bridge Telegram messages to Brig"
kind = "persistent"

[requires]
network = ["api.telegram.org"]
max_runtime = "forever"

[persistent]
rc_name = "brig_telegram"
entrypoint = "/usr/local/bin/brig-telegram"
restart_on_failure = true
depends_on = ["brig"]

[persistent.socket]
capabilities = ["submit_task", "read_status"]

[secrets]
telegram_token = { env = "BRIG_TELEGRAM_TOKEN" }
```

Note: `entrypoint` already points to `/usr/local/bin/brig-telegram`, which is
where the port will install the binary.

### brig's Makefile (reference)

```makefile
PREFIX?=    /usr/local
BINDIR=     ${PREFIX}/bin
MANDIR=     ${PREFIX}/share/man
SHAREDIR=   ${PREFIX}/share/brig
RCDIR=      ${PREFIX}/etc/rc.d

CARGO?=     cargo
CARGO_FLAGS=    --release

.PHONY: all build install clean test man

all: build

build:
    ${CARGO} build ${CARGO_FLAGS}

test:
    ${CARGO} test

install: build
    install -m 0755 target/release/brig ${BINDIR}/brig
    install -d ${SHAREDIR}/skills
    # ... man pages, rc.d script ...

clean:
    ${CARGO} clean

uninstall:
    rm -f ${BINDIR}/brig
    # ... man pages, rc.d script ...
```

### brig's rc.d script (reference)

```sh
#!/bin/sh
#
# PROVIDE: brig
# REQUIRE: LOGIN NETWORKING
# KEYWORD: shutdown
#
# Add the following lines to /etc/rc.conf to enable brig:
#
#   brig_enable="YES"
#   brig_user="jim"
#   brig_config="/home/jim/.brig/brig.conf"

. /etc/rc.subr

name=brig
rcvar=brig_enable

load_rc_config $name

: ${brig_enable:="NO"}
: ${brig_user:="root"}
: ${brig_config:="/usr/local/etc/brig.conf"}
: ${brig_flags:=""}

pidfile="/var/run/${name}.pid"
command="/usr/local/bin/brig"
command_args="-d -c ${brig_config} ${brig_flags}"

start_precmd="brig_prestart"

brig_prestart()
{
    local data_dir
    data_dir=$(grep '^data_dir=' "${brig_config}" 2>/dev/null | cut -d= -f2)
    data_dir="${data_dir:-/home/${brig_user}/.brig}"

    install -d -o "${brig_user}" -g "$(id -gn ${brig_user})" "${data_dir}"
    install -d -o "${brig_user}" -g "$(id -gn ${brig_user})" "${data_dir}/sock"
    install -d -o "${brig_user}" -g "$(id -gn ${brig_user})" "${data_dir}/skills"
    install -d -o "${brig_user}" -g "$(id -gn ${brig_user})" "${data_dir}/proposals"
}

run_rc_command "$1"
```

### enclave daemon rc.d (reference for daemon(8) wrapping)

brig-telegram is a foreground process (infinite poll loop).  The enclave
daemon rc.d template at `brig/templates/enclave_daemon.rc.d` shows the
daemon(8) pattern for backgrounding a foreground process in an rc.d script.
Use this as the model for the brig_telegram rc.d script.

## Deliverables

### 1. Makefile (BSD make)

Create `brig-telegram/Makefile` with these targets:

- `all` / `build` — `cargo build --release`
- `test` — `cargo test`
- `install` — installs binary and manifest only (jailed mode):
  - `install -m 0755 target/release/brig-telegram ${BINDIR}/brig-telegram`
  - `install -d ${SHAREDIR}/skills/telegram-gateway`
  - `install -m 0644 manifest.toml ${SHAREDIR}/skills/telegram-gateway/manifest.toml`
  - Print post-install message about `brig secret set` + `brig skill enable`
- `install-service` — installs the host-mode rc.d script:
  - `install -m 0755 scripts/rc.d/brig_telegram ${RCDIR}/brig_telegram`
  - Print message about rc.conf configuration
- `uninstall` — removes binary, manifest directory, rc.d script (if present)
- `clean` — `cargo clean`

Variables (matching brig's conventions):
```makefile
PREFIX?=    /usr/local
BINDIR=     ${PREFIX}/bin
SHAREDIR=   ${PREFIX}/share/brig
RCDIR=      ${PREFIX}/etc/rc.d
CARGO?=     cargo
CARGO_FLAGS=    --release
```

### 2. rc.d script for host mode

Create `brig-telegram/scripts/rc.d/brig_telegram`:

```sh
#!/bin/sh
#
# PROVIDE: brig_telegram
# REQUIRE: brig NETWORKING
# KEYWORD: shutdown
#
# Add the following lines to /etc/rc.conf to enable brig-telegram:
#
#   brig_telegram_enable="YES"
#   brig_telegram_token="your-bot-token"    # or use brig secret
#   brig_telegram_user="jim"                # user to run as
#
# Optional:
#   brig_telegram_socket="/var/brig/sock/brig.sock"
#   brig_telegram_name="telegram-gateway"
#   brig_telegram_prefix="tg"
#   brig_telegram_flags=""

. /etc/rc.subr

name=brig_telegram
rcvar=brig_telegram_enable

load_rc_config $name

: ${brig_telegram_enable:="NO"}
: ${brig_telegram_user:="root"}
: ${brig_telegram_token:=""}
: ${brig_telegram_socket:="/var/brig/sock/brig.sock"}
: ${brig_telegram_name:="telegram-gateway"}
: ${brig_telegram_prefix:="tg"}
: ${brig_telegram_flags:=""}

pidfile="/var/run/${name}.pid"
command="/usr/local/bin/brig-telegram"
command_args="${brig_telegram_flags}"

start_precmd="brig_telegram_prestart"

brig_telegram_prestart()
{
    if [ -z "${brig_telegram_token}" ]; then
        err 1 "brig_telegram_token is not set in /etc/rc.conf"
    fi
    export BRIG_TELEGRAM_TOKEN="${brig_telegram_token}"
    export BRIG_SOCKET="${brig_telegram_socket}"
    export BRIG_GATEWAY_NAME="${brig_telegram_name}"
    export BRIG_SESSION_PREFIX="${brig_telegram_prefix}"
}

run_rc_command "$1"
```

Key points:
- `REQUIRE: brig NETWORKING` — brig daemon must be running first
- `KEYWORD: shutdown` — clean stop on system shutdown
- Token is required; prestart fails if missing
- All env vars configurable via rc.conf

Note: brig-telegram runs in the foreground (infinite poll loop).  The rc.d
script needs to background it.  Check whether rc.subr handles this or
whether `daemon(8)` wrapping is needed.  If daemon wrapping is needed:

```sh
start_cmd="brig_telegram_start"

brig_telegram_start()
{
    brig_telegram_prestart || return 1
    /usr/sbin/daemon -f -p ${pidfile} -u ${brig_telegram_user} \
        /usr/bin/env \
        BRIG_TELEGRAM_TOKEN="${brig_telegram_token}" \
        BRIG_SOCKET="${brig_telegram_socket}" \
        BRIG_GATEWAY_NAME="${brig_telegram_name}" \
        BRIG_SESSION_PREFIX="${brig_telegram_prefix}" \
        ${command} ${command_args}
}
```

Look at `brig/templates/enclave_daemon.rc.d` for the daemon(8) pattern
already used in this project.

### 3. Port skeleton

Create `brig-telegram/port/` with the following files.  These are not
functional without a distfile URL and valid distinfo, but they establish
the structure for when the project is ready for the ports tree or a local
poudriere build.

**port/Makefile:**
```makefile
PORTNAME=       brig-telegram
DISTVERSION=    0.1.0
CATEGORIES=     net-im

MAINTAINER=     jim@example.com
COMMENT=        Telegram gateway for Brig
WWW=            https://github.com/jmspring/brig-telegram

LICENSE=        BSD2CLAUSE
LICENSE_FILE=   ${WRKSRC}/LICENSE

RUN_DEPENDS=    brig:sysutils/brig

USES=           cargo

PLIST_FILES=    bin/brig-telegram \
                share/brig/skills/telegram-gateway/manifest.toml

post-install:
    @${MKDIR} ${STAGEDIR}${PREFIX}/share/brig/skills/telegram-gateway
    ${INSTALL_DATA} ${WRKSRC}/manifest.toml \
        ${STAGEDIR}${PREFIX}/share/brig/skills/telegram-gateway/manifest.toml
```

**port/pkg-descr:**
```
Telegram Bot API gateway for Brig.  Bridges Telegram messages to Brig's
unix domain socket for LLM-driven task execution.

Synchronous.  No async runtime.  No Telegram bot framework.
```

**port/pkg-plist:**
```
bin/brig-telegram
share/brig/skills/telegram-gateway/manifest.toml
```

**port/pkg-message:**
```
[
{ type: install
  message: <<EOM
To use brig-telegram in jailed mode (recommended):

    brig secret set telegram-gateway.telegram_token
    brig skill enable telegram-gateway

To use brig-telegram as a host service instead:

    1. Copy the rc.d script:
       cp /usr/local/share/brig/skills/telegram-gateway/scripts/brig_telegram \
          /usr/local/etc/rc.d/brig_telegram

    2. Configure /etc/rc.conf:
       sysrc brig_telegram_enable=YES
       sysrc brig_telegram_token="your-bot-token"
       sysrc brig_telegram_user="jim"

    3. Start the service:
       service brig_telegram start

Do not run both modes simultaneously with the same session prefix.
EOM
}
]
```

**port/distinfo:**
```
# Placeholder — populate when release tarballs are available
# TIMESTAMP = 1713100000
# SHA256 (brig-telegram-0.1.0.tar.gz) = ???
# SIZE (brig-telegram-0.1.0.tar.gz) = ???
```

### 4. Update README.md

Add an "Install" section before "Manual run" that documents the BSD make
workflow:

```markdown
## Install

```sh
make                     # build release binary
sudo make install        # install binary + skill manifest
```

This installs:
- `/usr/local/bin/brig-telegram`
- `/usr/local/share/brig/skills/telegram-gateway/manifest.toml`

Then enable via brig (jailed):

```sh
brig secret set telegram-gateway.telegram_token
brig skill enable telegram-gateway
```

Or as a host service (no jail):

```sh
sudo make install-service    # install rc.d script
sudo sysrc brig_telegram_enable=YES
sudo sysrc brig_telegram_token="your-bot-token"
sudo sysrc brig_telegram_user="jim"
sudo service brig_telegram start
```
```

Replace the existing "Install as a Brig persistent skill" section.  The
"Manual run" section stays as-is for development/testing.

### 5. Update docs/GUIDE.md

The install section currently documents the manual `cargo build && cp` +
`brig skill add` + `brig skill enable` flow.  Update to show `make install`
as the primary path, with `brig skill enable` as the next step.  The manual
`brig skill add ./` path can remain as an alternative for development.

### 6. Create scripts/ directory

```
brig-telegram/scripts/
└── rc.d/
    └── brig_telegram
```

## File checklist

| File | Action |
|------|--------|
| `Makefile` | Create |
| `scripts/rc.d/brig_telegram` | Create |
| `port/Makefile` | Create |
| `port/pkg-descr` | Create |
| `port/pkg-plist` | Create |
| `port/pkg-message` | Create |
| `port/distinfo` | Create (placeholder) |
| `README.md` | Update install section |
| `docs/GUIDE.md` | Update install steps |

## Verification

- `make && sudo make install` puts binary and manifest in correct paths
- `brig skill list` shows `telegram-gateway` discovered at `/usr/local/share/brig/skills/`
- `brig skill enable telegram-gateway` succeeds (finds manifest, creates jail)
- `sudo make install-service` installs rc.d script
- `service brig_telegram start` works in host mode (with token configured)
- `make clean && make` rebuilds from scratch
- `sudo make uninstall` removes all installed files

## Notes

- The port skeleton in `port/` is not functional without distinfo.  It is
  scaffolding for future submission to the ports tree or local poudriere.
- The `USES=cargo` directive in the port Makefile handles Rust build
  integration with the ports framework (fetching crate dependencies,
  setting CARGO_HOME, etc.).
- Multi-bot setup (contrib manifests) continues to work — each instance
  needs its own `brig skill add --manifest contrib/manifest-ops.toml` etc.
  The port only installs the default single-bot manifest.
- The rc.d script needs to handle backgrounding.  brig-telegram is a
  foreground process (infinite getUpdates poll loop).  Check
  `brig/templates/enclave_daemon.rc.d` for the daemon(8) pattern.
