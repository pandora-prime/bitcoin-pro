.DEFAULT_GOAL := copy
BPRO_VERSION?=0.1.0

build:
	docker build -f bitcoin-pro-builder.dockerfile --build-arg=BPRO_VERSION=$(BPRO_VERSION) . -t bitcoin-pro:${BPRO_VERSION}

copy: build	
	docker create --name bitcoin-pro bitcoin-pro:$(BPRO_VERSION)
	docker cp bitcoin-pro:/bitcoin-pro ./bitcoin-pro_$(BPRO_VERSION)
	docker rm bitcoin-pro