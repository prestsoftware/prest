UIC_PY = $(patsubst %.ui,%.py,$(wildcard uic/*.ui))

.typecheck-ts: $(UIC_PY) *.py */*.py
	# (temporarily disabled)
	# mypy $(shell find . -name \*.py -not -path \*venv\* | grep -v uic/)
	touch .typecheck-ts

uic/%.py: uic/%.ui
	pyuic5 $< -o $@
