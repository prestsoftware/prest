UIC_PY = $(patsubst %.ui,%.py,$(wildcard uic/*.ui))

.typecheck-ts: $(UIC_PY) *.py */*.py
	venv/bin/python -m mypy --exclude /uic/ $(shell find . -name \*.py -not -path \*venv\* -not -path \*uic\* | grep -v uic/)
	touch .typecheck-ts

uic/%.py: uic/%.ui
	pyuic5 $< -o $@
