.DEFAULT_GOAL := help

.PHONY: help build test lint check metrics extract tree project-tree repo-tree clean

help:
	@.scripts/help.sh

build:
	@.scripts/build.sh

test:
	@.scripts/test.sh

lint:
	@.scripts/lint.sh

check: lint test

extract: build
	@target/release/topo extract $(REPO)

metrics:
	@.scripts/metrics.sh

tree: project-tree

project-tree:
	@bash .scripts/show-tree.sh project

repo-tree:
	@bash .scripts/show-tree.sh repo

clean:
	@.scripts/clean.sh

-include .local/scripts/local.mk
