.DEFAULT_GOAL := help

.PHONY: help build test lint check metrics clean

help:
	@.scripts/help.sh

build:
	@.scripts/build.sh

test:
	@.scripts/test.sh

lint:
	@.scripts/lint.sh

check: lint test

metrics:
	@.scripts/metrics.sh

clean:
	@.scripts/clean.sh
