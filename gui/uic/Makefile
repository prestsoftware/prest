UI = $(wildcard *.ui)
PY = $(UI:.ui=.py)

all: $(PY)

%.py: %.ui
	pyuic5 $< -o $@
