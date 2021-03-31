.DEFAULT_GOAL := start
TARGET?=linux
BPRO_VERSION?=0.1.0

build:
	docker build -f ${TARGET}-builder.dockerfile --build-arg=BPRO_VERSION=$(BPRO_VERSION) . -t bitcoin-pro:${BPRO_VERSION}
	yes | docker image prune --filter label=stage=builder

copy: build	
	docker create --name bitcoin-pro bitcoin-pro:$(BPRO_VERSION)
	docker cp bitcoin-pro:/root/.cargo/bin/bitcoin-pro ./bitcoin-pro_$(BPRO_VERSION)
	docker rm bitcoin-pro

start: build
	x11docker bitcoin-pro:${BPRO_VERSION}

.PHONY: build copy start
