.DEFAULT_GOAL := help

.PHONY: help build test lint check metrics index tree project-tree repo-tree functions function-index symbols definition imports overview hotspots hotspots-report visualize visualize-dependencies visualize-hotspots clean

help:
	@.scripts/help.sh

build:
	@.scripts/build.sh

test:
	@.scripts/test.sh

lint:
	@.scripts/lint.sh

check: lint test

index: build
	@bash .scripts/index.sh $(REPO)

extract: build
	@target/release/topo extract $(REPO)

metrics:
	@.scripts/metrics.sh

tree: project-tree

project-tree:
	@bash .scripts/show-tree.sh project

repo-tree:
	@bash .scripts/show-tree.sh repo

functions: function-index

function-index:
	@bash .scripts/show-functions.sh

symbols:
	@bash .scripts/show-symbols.sh "$(NAME)"

definition:
	@bash .scripts/show-definition.sh "$(SYMBOL)"

imports:
	@bash .scripts/show-imports.sh "$(FILE)"

overview:
	@bash .scripts/show-overview.sh

hotspots:
	@bash .scripts/visualize-hotspots.sh

hotspots-report:
	@bash .scripts/show-hotspots.sh

visualize:
	@bash .scripts/visualize-dependencies.sh

visualize-dependencies:
	@bash .scripts/visualize-dependencies.sh

visualize-hotspots:
	@bash .scripts/visualize-hotspots.sh

clean:
	@.scripts/clean.sh
