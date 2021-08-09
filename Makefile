OPAM=opam
EXEC=${OPAM} config exec
DUNE=${EXEC} dune --
PIN_DEPENDS=ocaml_freetype

.PHONY: build run

build:
	${DUNE} build @install
run:
	./_build/default/bin/main.exe

upgrade-pins:
	${OPAM} update -y --upgrade --quiet ${PIN_DEPENDS}
