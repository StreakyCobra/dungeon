IMAGE_NAME ?= localhost/dungeon
BUILD_TOOL ?= podman
GO_BUILD_OUTPUT ?= build/dungeon

BUILD_FLAGS :=
ifneq ($(NO_CACHE),)
BUILD_FLAGS += --no-cache
endif
ifneq ($(PULL),)
BUILD_FLAGS += --pull
endif

.PHONY: archlinux ubuntu image-% cli

archlinux: image-archlinux cli

ubuntu: image-ubuntu cli

image-%:
	$(BUILD_TOOL) build $(BUILD_FLAGS) -t $(IMAGE_NAME) -t $(IMAGE_NAME)-$* -f images/Containerfile.$* .

cli:
	mkdir -p $(dir $(GO_BUILD_OUTPUT))
	go build -o $(GO_BUILD_OUTPUT) ./cmd/dungeon
