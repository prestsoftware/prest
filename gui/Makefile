UIC_PY = $(patsubst %.ui,%.py,$(wildcard uic/*.ui))

.typecheck-ts: $(UIC_PY) *.py */*.py
	mypy $(shell find . -name \*.py | grep -v uic/)
	touch .typecheck-ts

uic/%.py: uic/%.ui
	pyuic5 $< -o $@
