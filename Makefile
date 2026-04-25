# Makefile for brig-telegram
# Uses BSD make conventions. Run with `make` on FreeBSD.

DESTDIR?=
PREFIX?=	/usr/local
BINDIR=		${DESTDIR}${PREFIX}/bin
SHAREDIR=	${DESTDIR}${PREFIX}/share/brig
RCDIR=		${DESTDIR}${PREFIX}/etc/rc.d

CARGO?=		cargo
CARGO_FLAGS=	--release

.PHONY: all build install install-service clean test uninstall

all: build

build:
	${CARGO} build ${CARGO_FLAGS}

test:
	${CARGO} test

install: build
	install -m 0755 target/release/brig-telegram ${BINDIR}/brig-telegram
	install -d ${SHAREDIR}/skills/telegram-gateway
	install -m 0644 manifest.toml ${SHAREDIR}/skills/telegram-gateway/manifest.toml
	@echo ""
	@echo "Installed. To enable in a brig jail:"
	@echo "  brig secret set telegram-gateway.telegram_token"
	@echo "  brig skill enable telegram-gateway"

install-service:
	install -m 0755 scripts/rc.d/brig_telegram ${RCDIR}/brig_telegram
	@echo ""
	@echo "rc.d script installed. Configure /etc/rc.conf:"
	@echo '  sysrc brig_telegram_enable=YES'
	@echo '  sysrc brig_telegram_token="your-bot-token"'
	@echo '  sysrc brig_telegram_user="jim"'

clean:
	${CARGO} clean

uninstall:
	rm -f ${BINDIR}/brig-telegram
	rm -rf ${SHAREDIR}/skills/telegram-gateway
	rm -f ${RCDIR}/brig_telegram
