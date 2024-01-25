#!/usr/bin/env python3

import sys
import logging
import argparse
import tqdm
from typing import Optional

import dataset.budgetary

from model import *
from dataset import load_raw_csv
from gui.progress import Worker, MockWorker
from gui.estimation import Options as EstimationOpts
from gui.estimation import DistanceScore
from dataset.experimental_data import ExperimentalData

logging.basicConfig(level=logging.DEBUG)
log = logging.getLogger(__name__)

class ProgressWorker(Worker):
    def __init__(self):
        self.bar : Optional[tqdm.tqdm] = None
        self.size = None
        self.last_value = 0

    def set_work_size(self, size : int) -> None:
        self.size = size
        self.bar = tqdm.tqdm(total=size)

    def set_progress(self, value : int) -> None:
        assert self.bar
        self.bar.update(value - self.last_value)
        self.last_value = value

def budgetary_consistency(args):
    ds = dataset.budgetary.load_from_csv(args.fname_in)
    dsc = ds.analysis_consistency(ProgressWorker(), None)
    variant = dsc._get_export_variant(args.export_variant)
    dsc.export(args.fname_out, '*.csv', variant, ProgressWorker())

def consistency_deterministic(args):
    rows = load_raw_csv(args.fname_in)
    ds = ExperimentalData.from_csv('dataset', rows[1:], (0, 1, None, 2))
    dsm = ds.analysis_consistency_deterministic(ProgressWorker(), None)
    variant = dsm._get_export_variant(args.export_variant)
    dsm.export(args.fname_out, '*.csv', variant, MockWorker())

def estimate(args):
    rows = load_raw_csv(args.fname_in)
    ds = ExperimentalData.from_csv('dataset', rows[1:], (0, 1, None, 2))

    AVAILABLE_MODELS = [
        preorder(strict=True, total=True),
        preorder(strict=False, total=True),
        unattractive(strict=True, total=True),
        unattractive(strict=False, total=True),
        preorder(strict=True, total=False),
        preorder(strict=False, total=False),
        UndominatedChoice(strict=True),
        UndominatedChoice(strict=False),
        PartiallyDominantChoice(fc=True),
        PartiallyDominantChoice(fc=False),
        Overload(PreorderParams(strict=True, total=True)),
        Overload(PreorderParams(strict=False, total=True)),
        StatusQuoUndominatedChoice(),
        TopTwo(),
        SequentiallyRationalizableChoice(),
        Swaps(),
    ]

    if not args.models:
        print('Please specify a model using -m:')
        for m in AVAILABLE_MODELS:
            print('  ' + str(m))

        sys.exit(1)

    if args.models == 'all':
        models = AVAILABLE_MODELS
    else:
        models = [
            m
            for m in AVAILABLE_MODELS
            if str(m) in args.models
        ]

    if not models:
        raise Exception('bad model spec')

    dsm = ds.analysis_estimation(ProgressWorker(), EstimationOpts(
        models=models,
        disable_parallelism=args.sequential,
        disregard_deferrals=args.disregard_deferrals,
        distance_score=DistanceScore.HOUTMAN_MAKS,
    ))
    variant = dsm._get_export_variant(args.export_variant)
    dsm.export(args.fname_out, '*.csv', variant, MockWorker())

def main(args):
    if args.action == 'estimate':
        estimate(args)
    elif args.action == 'consistency-deterministic':
        consistency_deterministic(args)
    elif args.action == 'budgetary':
        budgetary_consistency(args)
    else:
        raise Exception(f'unknown action: {args.action}')

if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest='action', help='subcommands')
    sub.required = True

    apE = sub.add_parser('estimate', help='model estimation')
    apE.add_argument('fname_in', metavar='input.csv')
    apE.add_argument('fname_out', metavar='output.csv')
    apE.add_argument('-e', dest='export_variant',
        default='compact (human-friendly)',
        help='export variant [%(default)s]',
    )
    apE.add_argument('-s', '--sequential', default=False, action='store_true', help='disable paralellism')
    apE.add_argument('-m', dest='models', metavar='MODEL', nargs='+', help='model(s)')
    apE.add_argument('--disregard-deferrals', default=False, action='store_true')

    apC = sub.add_parser('consistency-deterministic', help='general consistency (deterministic)')
    apC.add_argument('fname_in', metavar='input.csv')
    apC.add_argument('fname_out', metavar='output.csv')
    apC.add_argument('-e', dest='export_variant',
        default='Summary',
        help='export variant [%(default)s]',
    )

    apB = sub.add_parser('budgetary', help='budgetary consistency')
    apB.add_argument('fname_in', metavar='input.csv')
    apB.add_argument('fname_out', metavar='output.csv')
    apB.add_argument('-e', dest='export_variant',
        default='Summary',
        help='export variant [%(default)s]',
    )

    main(ap.parse_args())
