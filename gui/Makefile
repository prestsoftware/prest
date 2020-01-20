UIC_PY = $(patsubst %.ui,%.py,$(wildcard uic/*.ui))

uic/%.py: uic/%.ui
	pyuic5 $< -o $@

.typecheck-ts: $(UIC_PY) *.py */*.py
	mypy --platform win32 --ignore-missing-imports --check-untyped-defs --strict-optional main.py
	touch .typecheck-ts

.typecheck-ts-mac: $(UIC_PY) *.py */*.py
	mypy --ignore-missing-imports --check-untyped-defs --strict-optional main.py
	touch .typecheck-ts-mac
