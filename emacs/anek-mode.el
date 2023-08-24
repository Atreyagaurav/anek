;; have links like `anek:station/liberty' and then use that input to
;; render the link template to actually open the file. this way we
;; don't have to make multiple files with different links, but rather
;; I can reuse the same link to open multiple files when I
;; want. Useful for comparing between different anek inputs
(require 'ol)

(org-link-set-parameters "anek"
                         :follow #'org-anek-open
                         :export #'org-anek-export
                         :store #'org-anek-store-link)

(defcustom anek-command 'anek
  "The Command to be used to run anek program."
  :group 'org-link
  :type 'command)

(defvar anek-url-template "./.anek/inputs/"
  "Template to be used to open anek lists")

(defun set-anek-url-template (templ)
  (interactive "sEnter Template:")
  (setq anek-url-template templ))

(defun fill-anek-url-template (anek)
  (string-trim-right
   (shell-command-to-string
    (format "%s -q run -R '%s' -i %s"
	    anek-command
	    anek-url-template
	    anek))))

(defun org-anek-open (path)
  (find-file-other-window (fill-anek-url-template path)))
