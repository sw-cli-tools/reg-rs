;;; my-reg-rs.el --- reg-rs regression test integration for Emacs -*- lexical-binding: t; -*-

;; Author: Mike Wright
;; Version: 0.1.0
;; Package-Requires: ((emacs "28.1"))
;; Keywords: tools, processes, testing
;; URL: https://github.com/sw-cli-tools/reg-rs

;;; Commentary:

;; Emacs integration for reg-rs, a CLI regression testing tool.
;; Provides commands to run, list, show, update, reset, remove, and
;; create regression tests from within Emacs using compilation-mode.
;;
;; All commands run under the C-c r prefix:
;;   C-c r r   run all tests
;;   C-c r f   run with filter/args
;;   C-c r l   list tests
;;   C-c r s   show test details
;;   C-c r u   update baseline
;;   C-c r x   reset results
;;   C-c r d   delete test
;;   C-c r a   add test
;;   C-c r v   run verbose
;;   C-c r V   run very verbose
;;   C-c r R   rerun last command

;;; Code:

(require 'compile)
(require 'project)
(require 'subr-x)

;;;; Customization

(defgroup my-reg-rs nil
  "Run reg-rs regression tests from Emacs."
  :group 'tools
  :prefix "my/reg-rs-")

(defcustom my/reg-rs-shell "bash"
  "Shell used to invoke reg-rs aliases."
  :type 'string
  :group 'my-reg-rs)

(defcustom my/reg-rs-source
  "export PATH=\"$HOME/.local/softwarewrighter/bin:$PATH\" && source ~/github/sw-cli-tools/reg-rs/bin/source-rg.sh"
  "Shell fragment that sources reg-rs aliases.
Prepends the reg-rs binary location to PATH before sourcing."
  :type 'string
  :group 'my-reg-rs)

(defcustom my/reg-rs-buffer-name "*reg-rs*"
  "Default compilation buffer name for reg-rs output."
  :type 'string
  :group 'my-reg-rs)

;;;; State

(defvar my/reg-rs-last-command "rnrg"
  "Last reg-rs command that was executed.")

;;;; Root detection

(defun my/reg-rs-project-root ()
  "Return current project root, or `default-directory' if none."
  (if-let ((pr (project-current nil)))
      (expand-file-name (project-root pr))
    (expand-file-name default-directory)))

(defun my/reg-rs-root ()
  "Find the best root directory for reg-rs execution.
Checks for work/reg-rs directory, then .rgt/.tdb files,
then falls back to the project root."
  (or (locate-dominating-file default-directory "work/reg-rs")
      (locate-dominating-file
       default-directory
       (lambda (dir)
         (directory-files dir nil "\\.\\(rgt\\|tdb\\)\\'")))
      (my/reg-rs-project-root)))

;;;; Execution engine

(defun my/reg-rs--shell-command (cmd)
  "Wrap CMD in a shell invocation that sources reg-rs aliases."
  (format "%s -lc %S"
          my/reg-rs-shell
          (format "%s && %s" my/reg-rs-source cmd)))

(defun my/reg-rs--run (cmd &optional buffer-name)
  "Run reg-rs alias CMD from project root in compilation mode.
Optional BUFFER-NAME overrides the default buffer name."
  (setq my/reg-rs-last-command cmd)
  (let ((default-directory (my/reg-rs-root)))
    (compilation-start
     (my/reg-rs--shell-command cmd)
     'compilation-mode
     (lambda (_mode)
       (or buffer-name my/reg-rs-buffer-name)))))

;;;; Interactive commands

(defun my/reg-rs-run-all ()
  "Run all reg-rs tests in the current project."
  (interactive)
  (my/reg-rs--run "rnrg"))

(defun my/reg-rs-run (args)
  "Run rnrg with ARGS (filter pattern, flags, etc)."
  (interactive (list (read-string "rnrg args: ")))
  (my/reg-rs--run (string-trim (format "rnrg %s" args))))

(defun my/reg-rs-run-verbose ()
  "Run all reg-rs tests with verbose output."
  (interactive)
  (my/reg-rs--run "rnrg -v"))

(defun my/reg-rs-run-very-verbose ()
  "Run all reg-rs tests with full diff output."
  (interactive)
  (my/reg-rs--run "rnrg -vv"))

(defun my/reg-rs-list ()
  "List all reg-rs tests and their status."
  (interactive)
  (my/reg-rs--run "lsrg" "*reg-rs-list*"))

(defun my/reg-rs-show (args)
  "Show detailed test info via shrg with ARGS."
  (interactive (list (read-string "shrg args: ")))
  (my/reg-rs--run
   (string-trim (format "shrg %s" args))
   "*reg-rs-show*"))

(defun my/reg-rs-update (args)
  "Accept changed baseline via uprg with ARGS."
  (interactive (list (read-string "uprg args: ")))
  (my/reg-rs--run
   (string-trim (format "uprg %s" args))
   "*reg-rs-update*"))

(defun my/reg-rs-reset (args)
  "Reset test results via rsrg with ARGS."
  (interactive (list (read-string "rsrg args: ")))
  (my/reg-rs--run
   (string-trim (format "rsrg %s" args))
   "*reg-rs-reset*"))

(defun my/reg-rs-remove (args)
  "Remove test(s) via rmrg with ARGS."
  (interactive (list (read-string "rmrg args: ")))
  (my/reg-rs--run
   (string-trim (format "rmrg %s" args))
   "*reg-rs-remove*"))

(defun my/reg-rs-add (args)
  "Create a new test via adrg with ARGS."
  (interactive (list (read-string "adrg args: ")))
  (my/reg-rs--run
   (string-trim (format "adrg %s" args))
   "*reg-rs-add*"))

(defun my/reg-rs-rerun ()
  "Rerun the last reg-rs command."
  (interactive)
  (my/reg-rs--run my/reg-rs-last-command))

(defun my/reg-rs-help ()
  "Show reg-rs keybinding help in a temporary buffer."
  (interactive)
  (with-help-window "*reg-rs-help*"
    (princ "reg-rs commands (C-c r):\n\n")
    (princ "  r   Run all tests          (rnrg)\n")
    (princ "  f   Run with filter/args   (rnrg <args>)\n")
    (princ "  v   Run verbose            (rnrg -v)\n")
    (princ "  V   Run very verbose       (rnrg -vv)\n")
    (princ "  l   List tests             (lsrg)\n")
    (princ "  s   Show test details      (shrg)\n")
    (princ "  u   Update baseline        (uprg)\n")
    (princ "  x   Reset results          (rsrg)\n")
    (princ "  d   Delete test            (rmrg)\n")
    (princ "  a   Add/create test        (adrg)\n")
    (princ "  R   Rerun last command\n")
    (princ "  h   This help\n")))

;;;; Keybindings

(defvar my/reg-rs-map (make-sparse-keymap)
  "Keymap for reg-rs commands under C-c r.")

(define-key global-map (kbd "C-c r") my/reg-rs-map)

(define-key my/reg-rs-map (kbd "r") #'my/reg-rs-run-all)
(define-key my/reg-rs-map (kbd "f") #'my/reg-rs-run)
(define-key my/reg-rs-map (kbd "l") #'my/reg-rs-list)
(define-key my/reg-rs-map (kbd "s") #'my/reg-rs-show)
(define-key my/reg-rs-map (kbd "u") #'my/reg-rs-update)
(define-key my/reg-rs-map (kbd "x") #'my/reg-rs-reset)
(define-key my/reg-rs-map (kbd "d") #'my/reg-rs-remove)
(define-key my/reg-rs-map (kbd "a") #'my/reg-rs-add)
(define-key my/reg-rs-map (kbd "v") #'my/reg-rs-run-verbose)
(define-key my/reg-rs-map (kbd "V") #'my/reg-rs-run-very-verbose)
(define-key my/reg-rs-map (kbd "R") #'my/reg-rs-rerun)
(define-key my/reg-rs-map (kbd "h") #'my/reg-rs-help)

(provide 'my-reg-rs)

;;; my-reg-rs.el ends here
