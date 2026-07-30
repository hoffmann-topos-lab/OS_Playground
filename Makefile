# OS Playground — orquestra o app.
# "Compila tudo junto":  make all
#
# O projeto depende apenas do toolchain Rust: nada de runtime Python, nada de
# biblioteca de sistema. As animações são desenhadas pelo próprio app.
SHELL := /bin/bash

.PHONY: all icons build app run dev clean help

help:            ## Lista os alvos disponíveis
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
	  | awk 'BEGIN{FS=":.*?## "}{printf "  %-10s %s\n", $$1, $$2}'

all: icons build app   ## Gera os ícones, compila e monta o .app

icons:           ## Gera os ícones a partir de logo.png
	cargo run --bin make_icons
	iconutil -c icns assets/AppIcon.iconset -o assets/icons/AppIcon.icns

build:           ## Compila o app em modo release
	cargo build --release

app: build       ## Monta dist/"OS Playground".app (clicável no Finder)
	bash scripts/bundle_app.sh

run:             ## Executa o app (release)
	cargo run --release

dev:             ## Executa o app (debug, compila rápido)
	cargo run

clean:           ## Limpa artefatos de build
	cargo clean
	rm -rf dist
