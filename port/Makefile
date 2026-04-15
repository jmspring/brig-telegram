PORTNAME=	brig-telegram
DISTVERSION=	0.1.0
CATEGORIES=	net-im

MAINTAINER=	jim@example.com
COMMENT=	Telegram gateway for Brig
WWW=		https://github.com/jmspring/brig-telegram

LICENSE=	BSD2CLAUSE
LICENSE_FILE=	${WRKSRC}/LICENSE

RUN_DEPENDS=	brig:sysutils/brig

USES=		cargo

PLIST_FILES=	bin/brig-telegram \
		share/brig/skills/telegram-gateway/manifest.toml

post-install:
	@${MKDIR} ${STAGEDIR}${PREFIX}/share/brig/skills/telegram-gateway
	${INSTALL_DATA} ${WRKSRC}/manifest.toml \
	    ${STAGEDIR}${PREFIX}/share/brig/skills/telegram-gateway/manifest.toml
