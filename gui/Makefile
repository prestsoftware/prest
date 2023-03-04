UIC_PY = $(patsubst %.ui,%.py,$(wildcard uic/*.ui))

.typecheck-ts: $(UIC_PY) *.py */*.py
	# https://github.com/python/mypy/issues/13627
	poetry run mypy --exclude /uic/ $(shell find . -name \*.py -not -path \*venv\* -not -path \*uic\* | grep -v uic/)
	touch .typecheck-ts

uic/%.py: uic/%.ui
	poetry run pyuic5 $< -o $@
