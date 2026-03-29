;;; setup.el --- Load and configure my-reg-rs -*- lexical-binding: t; -*-

;;; Commentary:
;; Source this file from your Emacs init to enable reg-rs integration.
;;   (load "~/github/sw-cli-tools/reg-rs/elisp/setup.el")

;;; Code:

(load-file (expand-file-name "my-reg-rs.el"
                             (file-name-directory
                              (or load-file-name buffer-file-name))))

(provide 'my-reg-rs-setup)

;;; setup.el ends here
