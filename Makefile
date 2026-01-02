PG_CONFIG ?= /usr/lib/postgresql/17/bin/pg_config
BUILD_DIR ?= build
PGRX_HOME ?= /root/.pgrx
EXT_NAME := pg_rrf
DOCKER_COMPOSE ?= docker compose
DEV_SERVICE ?= dev

.PHONY: build test compose-build build-in-container test-in-container init-in-container clean

compose-build:
	$(DOCKER_COMPOSE) build

build: compose-build
	$(DOCKER_COMPOSE) run --rm $(DEV_SERVICE) bash -lc "make build-in-container"

test: compose-build
	$(DOCKER_COMPOSE) run --rm $(DEV_SERVICE) bash -lc "make test-in-container"

init-in-container:
	@if [ ! -d "$(PGRX_HOME)/pg17" ]; then \
		cargo pgrx init --pg17 $(PG_CONFIG); \
	fi

build-in-container: init-in-container
	cargo pgrx install --release --pg-config $(PG_CONFIG)
	@mkdir -p $(BUILD_DIR)
	@sharedir=$$($(PG_CONFIG) --sharedir); \
	pkglibdir=$$($(PG_CONFIG) --pkglibdir); \
	cp $$pkglibdir/$(EXT_NAME).so $(BUILD_DIR)/; \
	cp $$sharedir/extension/$(EXT_NAME).control $(BUILD_DIR)/; \
	cp $$sharedir/extension/$(EXT_NAME)--*.sql $(BUILD_DIR)/

test-in-container: init-in-container
	@pgdata=/tmp/pgrx-test-pgdata; \
	rm -rf $$pgdata; \
	mkdir -p $$pgdata; \
	chown -R postgres:postgres $$pgdata; \
	cargo pgrx test pg17 --runas postgres --pgdata $$pgdata

clean:
	rm -rf $(BUILD_DIR)
