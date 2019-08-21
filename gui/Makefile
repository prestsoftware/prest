UIC_PY = $(patsubst %.ui,%.py,$(wildcard uic/*.ui))

uic/%.py: uic/%.ui
	pyuic5 $< -o $@

.typecheck-ts: $(UIC_PY) *.py */*.py
	mypy --ignore-missing-imports --check-untyped-defs --strict-optional --warn-unreachable --warn-redundant-casts --strict-equality main.py
	touch .typecheck-ts
