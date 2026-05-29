i18n:
	./scripts/i18n/build.sh

i18n-check:
	find locale -type f -name '*.po' -print0 | xargs -0 -n1 msgfmt --check
