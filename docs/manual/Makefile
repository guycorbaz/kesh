# Makefile — build des manuels Kesh LaTeX
# Compile via xelatex (fontspec requis)
#
# Usage :
#   make             — build tous les PDF FR (canoniques)
#   make fr          — idem (alias)
#   make admin       — build uniquement le manuel administrateur FR
#   make user        — build uniquement le manuel utilisateur FR
#   make brochure    — build uniquement la brochure marketing FR
#   make all-langs   — build tous les manuels dans toutes les langues disponibles (FR + traductions)
#   make clean       — supprime les fichiers temporaires LaTeX
#   make distclean   — supprime aussi les PDF générés
#
# Prérequis :
#   - xelatex (TeX Live ≥ 2021 recommandé)
#   - Polices : TeX Gyre Heros (inclus par défaut). Optionnel : Inter + JetBrains Mono
#     (sudo apt install fonts-inter fonts-jetbrains-mono sur Debian/Ubuntu)

LATEX        := xelatex
LATEX_FLAGS  := -interaction=nonstopmode -halt-on-error -file-line-error
PASSES       := 2  # 2 passes pour résoudre toc + références croisées

LANGS_AVAILABLE := fr
LANGS_FUTURE    := de it en

DOCS := admin-manual user-manual marketing-brochure

# Cibles par défaut : tous les PDF FR
.PHONY: all fr admin user brochure all-langs clean distclean help

all: fr

help:
	@echo "Cibles Makefile manuels Kesh :"
	@echo "  make             — build tous les PDF FR"
	@echo "  make fr          — alias"
	@echo "  make admin       — manuel administrateur FR"
	@echo "  make user        — manuel utilisateur FR"
	@echo "  make brochure    — brochure marketing FR"
	@echo "  make all-langs   — toutes les langues disponibles"
	@echo "  make clean       — supprime fichiers temporaires"
	@echo "  make distclean   — supprime aussi les PDF"

fr: fr/admin-manual.pdf fr/user-manual.pdf fr/marketing-brochure.pdf

admin: fr/admin-manual.pdf

user: fr/user-manual.pdf

brochure: fr/marketing-brochure.pdf

all-langs:
	@for lang in $(LANGS_AVAILABLE); do \
		echo "═══ Build langue $$lang ═══"; \
		for doc in $(DOCS); do \
			if [ -f "$$lang/$$doc.tex" ]; then \
				$(MAKE) "$$lang/$$doc.pdf"; \
			else \
				echo "Skip $$lang/$$doc.tex (non créé, voir $$lang/README.md)"; \
			fi; \
		done; \
	done
	@echo ""
	@echo "Langues futures (stubs) :"
	@for lang in $(LANGS_FUTURE); do \
		echo "  - $$lang/ : voir $$lang/README.md pour ajouter la traduction"; \
	done

# Règle générique : *.tex → *.pdf via xelatex (2 passes)
%.pdf: %.tex shared/kesh-style.sty shared/kesh-logo.svg
	@echo "═══ Building $@ (2 passes) ═══"
	cd $(dir $<) && $(LATEX) $(LATEX_FLAGS) -output-directory=. $(notdir $<)
	cd $(dir $<) && $(LATEX) $(LATEX_FLAGS) -output-directory=. $(notdir $<)
	@echo "✓ $@ généré"

# Cleanup
clean:
	@find . -type f \( \
		-name '*.aux' -o \
		-name '*.log' -o \
		-name '*.out' -o \
		-name '*.toc' -o \
		-name '*.lof' -o \
		-name '*.lot' -o \
		-name '*.fls' -o \
		-name '*.fdb_latexmk' -o \
		-name '*.synctex.gz' -o \
		-name '*.bbl' -o \
		-name '*.blg' -o \
		-name '*.idx' -o \
		-name '*.ind' -o \
		-name '*.ilg' \
	\) -delete
	@echo "✓ Fichiers temporaires LaTeX supprimés"

distclean: clean
	@find . -name '*.pdf' -type f -delete
	@echo "✓ PDF supprimés"
