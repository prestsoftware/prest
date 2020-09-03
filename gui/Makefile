UIC_PY = $(patsubst %.ui,%.py,$(wildcard uic/*.ui))

uic/%.py: uic/%.ui
	pyuic5 $< -o $@

.typecheck-ts: $(UIC_PY) *.py */*.py
	mypy $(shell find -name \*.py)
	touch .typecheck-ts
