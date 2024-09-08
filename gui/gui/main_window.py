import sys
import logging
import functools
import os.path

from PyQt5.QtCore import Qt, pyqtSlot
from PyQt5.QtGui import QIcon, QKeySequence
from PyQt5.QtWidgets import QHeaderView, QMainWindow, QDialog, QMessageBox, \
    QTableWidgetItem, QFileDialog, QTreeWidgetItem, QProgressDialog, QMenu, QAction, \
    QStyle, QShortcut

import uic.main_window

import doc
import dataset
import workspace
import simulation
import platform_specific
import dataset.experimental_data
import dataset.budgetary
from core import Core
from gui.progress import Worker, Cancelled
from typing import Optional, List, Tuple, Any, Set, Dict, Iterator, Iterable, Callable

import gui
import gui.about
import gui.simulation
import gui.estimation
import gui.import_csv
import gui.subject_filter

log = logging.getLogger(__name__)

class MainWindowError(Exception):
    pass

class MainWindow(QMainWindow, uic.main_window.Ui_MainWindow, gui.ExceptionDialog):
    hidden_features_enabled : bool = False

    def __init__(self):

        # window setup
        log.debug('creating GUI')
        QMainWindow.__init__(self)
        self.setupUi(self)

        # icon
        self.setWindowIcon(QIcon(platform_specific.get_embedded_file_path(
            'images/prest.ico',  # deployment
            'gui/images/prest.ico',  # development
        )))

        # instance attributes
        self.workspace = workspace.Workspace()

        # main menu
        self.actionGenerate_random_subjects.triggered.connect(self.catch_exc(self.dlg_simulation))
        self.actionWorkspaceClear.triggered.connect(self.catch_exc(self.dlg_workspace_clear))
        self.actionWorkspaceLoad.triggered.connect(self.catch_exc(self.dlg_workspace_load))
        self.actionWorkspaceSave.triggered.connect(self.catch_exc(self.dlg_workspace_save))
        self.actionWorkspaceSaveAs.triggered.connect(self.catch_exc(self.dlg_workspace_save_as))
        self.actionDatasetImport.triggered.connect(self.catch_exc(self.dlg_dataset_import))
        self.actionImport_budgetary_dataset.triggered.connect(self.catch_exc(self.dlg_budgetary_import))
        self.actionHelp.triggered.connect(self.catch_exc(self.dlg_help_show))
        self.actionAbout_Prest.triggered.connect(self.catch_exc(self.dlg_about))
        self.actionQuit.triggered.connect(self.close)  # self.close is a slot -> not wrapping

        # debug
        self.actionShow_console_window.toggled.connect(self.catch_exc(self.show_console_window))
        self.actionCrash_core.triggered.connect(self.catch_exc(self.dlg_crash_core))
        self.actionSoft_core_failure.triggered.connect(self.catch_exc(self.dlg_soft_core_failure))
        self.actionHidden_features.toggled.connect(self.catch_exc(self.enable_hidden_features))

        self.tblDataSets.doubleClicked.connect(self.catch_exc(self.dlg_view_current_dataset))
        self.tblDataSets.customContextMenuRequested.connect(self.catch_exc(self.context_menu))

        self.tblDataSets.horizontalHeader().setSectionResizeMode(QHeaderView.ResizeToContents)
        self.tblDataSets.horizontalHeader().setStretchLastSection(False)

        if not platform_specific.is_windows():
            self.actionShow_console_window.setEnabled(False)

        self.enableDebuggingTools = QShortcut(
            QKeySequence(Qt.CTRL + Qt.Key_D),
            self,
        )
        self.enableDebuggingTools.activated.connect(self.enable_debugging_tools)
        self.menuDebugging_tools.menuAction().setVisible(False)

        try:
            doc.start_daemon(
                platform_specific.get_embedded_file_path(
                    'html',  # deployment
                    'docs/build/html',  # development
                    criterion=os.path.isdir,
                )
            )
        except OSError as e:
            log.exception('could not start doc server')
            # it's running elsewhere

    def enable_debugging_tools(self) -> None:
        log.debug('enabling debugging tools...')
        self.menuDebugging_tools.menuAction().setVisible(True)

    def dlg_about(self, _flag) -> None:
        gui.about.About().exec_()

    def dlg_budgetary_import(self, _flag) -> None:
        fname, _ = QFileDialog.getOpenFileName(
            self,
            "Import budgetary dataset",
            filter="CSV files (*.csv)"
        )

        if not fname:
            return

        ds = dataset.budgetary.load_from_csv(fname)
        self.add_dataset(ds)

    def dlg_help_show(self, _flag) -> None:
        doc.open_in_browser('index.html')

    def context_menu(self, pos) -> None:
        # item is Optional[int] in some versions of PyQt
        # but non-optional QTableWidgetItem in other versions of PyQt
        # so we do this to make it work anywhere
        item : Optional[Any] = self.tblDataSets.itemAt(pos)
        if item is None:
            log.debug('context menu requested but no item selected')
            return

        ds = self.workspace.datasets[
            self.tblDataSets.row(item)
        ]


        menu = QMenu(self)
        icon_hidden = QIcon(platform_specific.get_embedded_file_path(
            'images/experimental.png',      # deployment
            'gui/images/experimental.png',  # development
        ))

        a_view = QAction("View...", menu)
        a_view.triggered.connect(self.catch_exc(ds.dlg_view))
        a_view.setStatusTip('Display the dataset in a separate window. Also available via double click.')
        menu.addAction(a_view)

        analyses = ds.get_analyses()
        if analyses:
            m_analyses = QMenu(menu)
            a_analyses = QAction("Analysis", menu)
            a_analyses.setMenu(m_analyses)
            a_analyses.setStatusTip('Run various analyses on the dataset.')
            menu.addAction(a_analyses)

            icon : Optional[QIcon]
            for analysis in analyses:
                if analysis.is_hidden:
                    if self.hidden_features_enabled:
                        icon = icon_hidden
                    else:
                        # skip this one
                        continue
                else:
                    icon = None

                def mkanalyse(analysis):
                    def analyse(_flag):
                        new_ds = ds.analyse(analysis, self)
                        if new_ds is not None:
                            self.add_dataset(new_ds)

                    return analyse

                if analysis.config:
                    analysis_name = analysis.name + '...'
                else:
                    analysis_name = analysis.name

                a_analysis = QAction(analysis_name, m_analyses)
                if icon:
                    a_analysis.setIcon(icon)

                a_analysis.analyse = mkanalyse(analysis)  # type: ignore
                a_analysis.triggered.connect(self.catch_exc(a_analysis.analyse))  # type: ignore
                m_analyses.addAction(a_analysis)

        export_variants = ds.get_export_variants()
        if export_variants:
            m_exports = QMenu(menu)
            a_exports = QAction("Export", menu)
            a_exports.setMenu(m_exports)
            a_exports.setStatusTip('Export the dataset into a file on disk.')
            menu.addAction(a_exports)

            for export_variant in export_variants:
                def mkexport(export_variant):
                    def export(_flag):
                        fname, fformat = QFileDialog.getSaveFileName(
                            self,
                            "Export dataset",
                            filter="Excel 2010+ (*.xlsx);;Comma Separated Values (*.csv)"
                        )
                        if not fname:
                            return

                        class MyWorker(Worker):
                            def work(self):
                                ds.export(fname, fformat, export_variant, self)

                        try:
                            MyWorker().run_with_progress(None, "Exporting to %s..." % fname)
                        except Cancelled:
                            log.info('export cancelled')

                    return export

                a_export = QAction(export_variant.name + '...', m_exports)
                export = mkexport(export_variant)
                a_export.triggered.connect(self.catch_exc(export))
                m_exports.addAction(a_export)

        menu.addSeparator()

        def delete(_flag):
            answer = QMessageBox.question(
                self,
                "Delete dataset",
                "Do you really want to delete the selected dataset?",
                defaultButton=QMessageBox.No
            )

            if answer == QMessageBox.Yes:
                self.delete_dataset(ds)

        a_delete = QAction("Delete", menu)
        a_delete.delete = delete  # type: ignore
        a_delete.triggered.connect(self.catch_exc(a_delete.delete))  # type: ignore
        a_delete.setStatusTip('Remove the dataset from the workspace. Asks for confirmation.')
        menu.addAction(a_delete)

        menu.popup(self.tblDataSets.viewport().mapToGlobal(pos))

    def closeEvent(self, event) -> None:
        if self.isWindowModified():
            answer = QMessageBox.warning(
                self,
                "Quit Prest",
                "Do you want to save the current workspace before quitting?",
                QMessageBox.StandardButtons(QMessageBox.Yes | QMessageBox.No | QMessageBox.Cancel),
                QMessageBox.Cancel,
            )
            if answer == QMessageBox.Cancel:
                event.ignore()
                return
            elif answer == QMessageBox.Yes:
                saved = self.dlg_workspace_save()
                if not saved:
                    QMessageBox.information(
                        self,
                        "Quit Prest",
                        "The save dialog was cancelled, not quitting.",
                    )
                    event.ignore()
                    return

        event.accept()

    def dlg_workspace_clear(self, _flag) -> None:
        if self.isWindowModified():
            answer = QMessageBox.warning(
                self,
                "Clear workspace",
                "Do you want to save the current workspace before clearing it?",
                QMessageBox.StandardButtons(QMessageBox.Yes | QMessageBox.No | QMessageBox.Cancel),
                QMessageBox.Cancel,
            )
            if answer == QMessageBox.Cancel:
                return
            elif answer == QMessageBox.Yes:
                saved = self.dlg_workspace_save()
                if not saved:
                    QMessageBox.information(
                        self,
                        "Clear workspace",
                        "The save dialog was cancelled, not clearing the workspace.",
                    )
                    return

        self.workspace.datasets = []
        self.setWindowModified(False)
        self.setWindowFilePath('')
        self.refresh_datasets()

    def dlg_workspace_load(self, _flag) -> None:
        if self.isWindowModified():
            answer = QMessageBox.warning(
                self,
                "Load workspace",
                "Do you want to save the current workspace before loading a new one?",
                QMessageBox.StandardButtons(QMessageBox.Yes | QMessageBox.No | QMessageBox.Cancel),
                QMessageBox.Cancel,
            )
            if answer == QMessageBox.Cancel:
                return
            elif answer == QMessageBox.Yes:
                saved = self.dlg_workspace_save()
                if not saved:
                    QMessageBox.information(
                        self,
                        "Load workspace",
                        "The save dialog was cancelled, not switching the workspace.",
                    )
                    return

        fname, _ftype = QFileDialog.getOpenFileName(
            self,
            "Load workspace",
            ".",
            "Prest Workspace File (*.pwf);;All files (*.*)",
        )
        if fname:
            class MyWorker(Worker):
                def work(self, workspace):
                    workspace.load_from_file(self, fname)

            try:
                MyWorker(self.workspace).run_with_progress(self, "Loading %s..." % fname)
                self.refresh_datasets()
                self.setWindowFilePath(fname)
                self.setWindowModified(False)
            except Cancelled:
                log.info('PWF load cancelled')

    def workspace_save_into(self, fname : str) -> bool:
        class MyWorker(Worker):
            def work(self, workspace):
                workspace.save_to_file(self, fname)

        try:
            MyWorker(self.workspace).run_with_progress(self, 'Saving to %s...' % fname)
            self.setWindowFilePath(fname)
            self.setWindowModified(False)
            return True
        except Cancelled:
            log.info('PWF save cancelled')
            return False

    # returns bool because it's used in workspace_load
    # True: saved successfully
    # False: save cancelled
    def dlg_workspace_save(self, _flag=None) -> bool:
        if self.windowFilePath():
            # got a filename, just save
            self.workspace_save_into(self.windowFilePath())
            self.setWindowModified(False)
            return True
        else:
            # no filename, pass on to "Save as..."
            return self.dlg_workspace_save_as()

    # returns bool because it's used in workspace_load
    # True: saved successfully
    # False: save cancelled
    def dlg_workspace_save_as(self, _flag=None) -> bool:
        fname, _ftype = QFileDialog.getSaveFileName(
            self,
            "Save workspace",
            "workspace.pwf",
            "Prest Workspace File (*.pwf);;All files (*.*)",
        )
        if fname:
            return self.workspace_save_into(fname)
        else:
            return False

    def dlg_simulation(self, _flag):
        dlg = gui.simulation.Simulation(self.hidden_features_enabled)
        if dlg.exec() != QDialog.Accepted:
            return

        options = dlg.value()

        class MyWorker(Worker):
            def work(self) -> dataset.experimental_data.ExperimentalData:
                self.set_work_size(options.subject_count)
                ds = dataset.experimental_data.ExperimentalData(options.dataset_name, [])
                ds.alternatives = options.alternatives
                ds.observ_count = 0

                with Core() as core:
                    self.interrupt = lambda: core.shutdown()

                    for subj_nr in range(1, options.subject_count+1):
                        while True:
                            response = simulation.run(core, simulation.Request(
                                name='random%d' % subj_nr,
                                alternatives=options.alternatives,
                                gen_menus=options.gen_menus,
                                gen_choices=options.gen_choices,
                                preserve_deferrals=False,
                            ))

                            subject_accepted = gui.subject_filter.accepts(
                                options.subject_filter,
                                core,
                                response.subject_packed,
                            )

                            if subject_accepted:
                                ds.subjects.append(response.subject_packed)
                                ds.observ_count += response.observation_count
                                self.set_progress(subj_nr)
                                # move on to the next subject
                                break
                            else:
                                # retry
                                continue

                return ds

        try:
            new_ds = MyWorker().run_with_progress(self, 'Generating subjects...')
            self.add_dataset(new_ds)
        except Cancelled:
            log.debug('simulation cancelled')

    def show_console_window(self, should_show: bool):
        if should_show:
            platform_specific.show_console()
        else:
            platform_specific.hide_console()

    def enable_hidden_features(self, enable: bool):
        self.hidden_features_enabled = enable
        self.actionGenerate_subjects_with_filtering.setVisible(True)

    def shutdown(self):
        log.debug('shutting GUI down')
        # nothing else to do

    def dlg_view_current_dataset(self, _flag):
        ds = self.selected_dataset()
        if ds is not None:
            ds.dlg_view()

    def dlg_crash_core(self, _flag):
        with Core() as core:
            core.crash()

    def dlg_soft_core_failure(self, _flag):
        with Core() as core:
            core.soft_failure()
        
    def dlg_dataset_import(self, _flag):
        dlg = gui.import_csv.ImportCsv(self)
        dlg.run()

    def add_dataset(self, ds : dataset.Dataset):
        assert len(self.workspace.datasets) == self.tblDataSets.rowCount()

        # insert into datasets
        self.workspace.datasets.append(ds)
        self.setWindowModified(True)

        self.add_dataset_to_table(ds)

    def delete_dataset(self, ds : dataset.Dataset):
        assert len(self.workspace.datasets) == self.tblDataSets.rowCount()

        idx = self.workspace.datasets.index(ds)
        self.tblDataSets.removeRow(idx)
        self.workspace.datasets.pop(idx)
        self.setWindowModified(True)

    def add_dataset_to_table(self, ds : dataset.Dataset):
        # insert into the table widget
        tbl = self.tblDataSets
        j = tbl.rowCount()
        tbl.insertRow(j)
        tbl.setItem(j, 0, QTableWidgetItem(ds.label_name()))
        #tbl.setItem(j, 1, QTableWidgetItem(ds.label_type()))
        item_alts = QTableWidgetItem(ds.label_alts())
        item_alts.setTextAlignment(Qt.AlignRight | Qt.AlignVCenter)  # type: ignore
        tbl.setItem(j, 1, item_alts)
        tbl.setItem(j, 2, QTableWidgetItem(ds.label_size()))

        tbl.setCurrentCell(j, 0)

    def refresh_datasets(self):
        self.tblDataSets.setRowCount(0)
        for ds in self.workspace.datasets:
            self.add_dataset_to_table(ds)

    def selected_dataset(self):
        if not self.workspace.datasets:
            return None

        idx = self.tblDataSets.currentRow()

        try:
            return self.workspace.datasets[idx]
        except IndexError as e:
            raise MainWindowError('Internal error: selecting non-existent dataset #%d' % idx) from e
