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

(defvar anek-url-template nil
  "Template to be used to open anek lists")

(defun set-anek-url-template (templ)
  (interactive "sEnter Template:")
  (setq anek-url-template templ))

(defun anek-run-command (args)
  (string-trim-right
   (shell-command-to-string
    (format "%s -q %s"
	    anek-command
	    args))))

(defun fill-anek-url-template (anek)
  (anek-run-command
    (format "run -R '%s' -i %s"
	    anek-url-template
	    anek)))

(defun anek-list (flag)
  (split-string (anek-run-command flag) "\n"))

(defun org-anek-open (path)
  (if anek-url-template
      (find-file-other-window (fill-anek-url-template path))
    (message "Set Anek Url Template first.")))

(defun anek-set-url-from-file (template)
  (interactive (list
		(let ((file (read-file-name "Select a file: ")))
		  (read-string "Anek URL Template: " file))))
  (set-anek-url-template template))

(defun anek-links-buffer (batch-file)
  (interactive (list (completing-read
		      "Batch File:"
		      (anek-list "list -b"))))
  (let ((files (anek-list (concat "edit -e batch/" batch-file)))
	(buf (get-buffer-create (format "*Anek [%s]*" batch-file))))
    (with-current-buffer buf
      (insert "Links for " batch-file "\n")
      (cl-loop
       for file in files
       do (insert (format "[[anek:%s][%s]]\n" file file)))
      (org-mode))
    (switch-to-buffer buf)))
