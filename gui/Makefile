uic/%.py: uic/%.ui
	pyuic5 $< -o $@

.typecheck-ts: $(UIC_PY) *.py */*.py
	mypy --platform win32 --ignore-missing-imports --check-untyped-defs --strict-optional main.py
	touch .typecheck-ts
