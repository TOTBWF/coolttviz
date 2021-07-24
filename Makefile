OPAM=opam
EXEC=${OPAM} config exec
DUNE=${EXEC} dune --

.PHONY: build run

build:
	${DUNE} build @install
run:
	./_build/default/bin/main.exe
