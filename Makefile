.DEFAULT_GOAL := help

# The release binary, wherever CARGO_TARGET_DIR puts it.
export TOPO_BIN := $(or $(CARGO_TARGET_DIR),target)/release/topo

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
	@$(TOPO_BIN) extract $(REPO)

metrics: build
	@.scripts/metrics.sh

tree: project-tree

project-tree: build
	@bash .scripts/show-tree.sh project

repo-tree: build
	@bash .scripts/show-tree.sh repo

clean:
	@.scripts/clean.sh

-include .local/scripts/local.mk
